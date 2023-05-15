use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use rand::Rng;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::env::args;
use std::time::Duration;
use tokio::{task, time};

use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, sql::Thing, Surreal};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000")
        .await
        .expect("Unable to connect to database");

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .unwrap();

    db.use_ns("main").use_db("main").await.unwrap();

    // let mut mqttoptions = MqttOptions::new("pv", "kruepv.gibip.de", 11883);
    let mut mqttoptions = MqttOptions::new("#123ae359", "mqtt.eclipseprojects.io", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut connection) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("/i40/#", QoS::AtMostOnce).await.unwrap();

    let testing = args().skip(1).next();
    match testing {
        Some(t) => {
            if t.to_lowercase() == "testing".to_owned() {
                println!("Running in test mode!");
                task::spawn(async move {
                    loop {
                        for sensor in mock().iter() {
                            let num: i32 = rand::thread_rng().gen_range(0..1000);
                            client
                                .publish(*sensor, QoS::AtLeastOnce, false, num.to_string())
                                .await
                                .unwrap();
                        }
                        time::sleep(Duration::from_secs(5)).await;
                    }
                });
            }
        }
        None => {
            println!("Running in production mode!");
        }
    }

    loop {
        let notification = connection.poll().await.unwrap();
        match notification {
            rumqttc::Event::Incoming(incoming) => match incoming {
                rumqttc::Packet::Publish(published) => {
                    let val = Vec::from(published.payload);
                    let val = String::from_utf8(val).unwrap();

                    let x = published.topic.replace("/i40/fertigungsanlage/", "");
                    let mut data = x.splitn(2, "/");

                    let station_name = data.next();
                    let sensor_name = data.next().unwrap().replace("/", "_");

                    let record: Option<Record> = db
                        .query("SELECT * FROM sensor WHERE display_name = $name")
                        .bind(("name", &sensor_name))
                        .await
                        .unwrap()
                        .take(0)
                        .unwrap();

                    match record {
                        Some(r) => {
                            println!("found record! inserting; {:?}", &r.id);
                            let res: Record = db
                                .create("sensor_value")
                                .content(SensorValue {
                                    sensor: r.id.clone(),
                                    value: val,
                                    server_timestamp: Utc::now(),
                                })
                                .await
                                .unwrap();
                            db.query("RELATE $sensor->hasValue->$sensor_value")
                                .bind(("sensor", &r.id))
                                .bind(("sensor_value", &res.id))
                                .await
                                .unwrap();
                        }
                        None => {
                            let station: Option<Record> = db
                                .query("SELECT id FROM station WHERE name = $name")
                                .bind(("name", &station_name.unwrap()))
                                .await
                                .unwrap()
                                .take(0)
                                .unwrap_or(None);

                            if station.is_some() {
                                let _: Record = db
                                    .create("sensor")
                                    .content(CreateSensor {
                                        display_name: sensor_name,
                                        station: station.unwrap().id,
                                    })
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                }
                _ => {}
            },
            rumqttc::Event::Outgoing(_) => {}
        }
    }

    // Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Record {
    id: Thing,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CreateSensor {
    display_name: String,
    station: Thing,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SensorValue {
    value: String,
    sensor: Thing,
    #[serde(with = "ts_seconds")]
    server_timestamp: DateTime<Utc>,
}

fn mock() -> [&'static str; 12] {
    [
        "/i40/fertigungsanlage/palettenlager/dosenfuellstand",
        "/i40/fertigungsanlage/palettenlager/kugelfuellstand/rot",
        "/i40/fertigungsanlage/palettenlager/kugelfuellstand/gruen",
        "/i40/fertigungsanlage/palettenlager/kugelfuellstand/blau",
        "/i40/fertigungsanlage/palettenlager/deckelfuellstand/rot",
        "/i40/fertigungsanlage/palettenlager/deckelfuellstand/gruen",
        "/i40/fertigungsanlage/palettenlager/deckelfuellstand/blau",
        "/i40/fertigungsanlage/palettenlager/palettenfuellstandrandom",
        "/i40/fertigungsanlage/presswerk/arm/motorgeschwindigkeit/x",
        "/i40/fertigungsanlage/presswerk/arm/motorgeschwindigkeit/y",
        "/i40/fertigungsanlage/presswerk/arm/motorgeschwindigkeit/z",
        "/i40/fertigungsanlage/presswerk/presse/pressenstatus",
    ]
}
