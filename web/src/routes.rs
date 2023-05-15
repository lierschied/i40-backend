use crate::middleware::authorization::JWTAuthorization;
use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};

pub fn config(app: &mut web::ServiceConfig) {
    app.service(
        web::scope("/api/v1")
            .wrap(JWTAuthorization)
            .service(crate::auth::decode)
            .service(get_stations)
            .service(get_sensor_values)
            .service(get_sensor)
            .service(get_sensors)
            .service(crate::api::get_measurments)
            .service(crate::api::get_alarms),
    );
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Sensor {
    id: Thing,
    display_name: String,
    values: Option<Vec<SensorValue>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct SensorValue {
    id: Thing,
    sensor: Thing,
    value: String,
    server_timestamp: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Station {
    id: Thing,
    name: String,
}

#[get("stations")]
async fn get_stations(db: web::Data<Surreal<Client>>) -> HttpResponse {
    let stations: Vec<Station> = db.select("station").await.unwrap();
    HttpResponse::Ok().json(stations)
}

#[get("/sensor/{sensor}")]
async fn get_sensor(sensor: web::Path<String>, db: web::Data<Surreal<Client>>) -> HttpResponse {
    let sensor: Option<Sensor> = db
        .query("SELECT * FROM sensor WHERE display_name = $name")
        .bind(("name", sensor.into_inner()))
        .await
        .unwrap()
        .take(0)
        .unwrap();
    HttpResponse::Ok().json(sensor)
}

#[get("/station/{station}/sensors")]
async fn get_sensors(station: web::Path<String>, db: web::Data<Surreal<Client>>) -> HttpResponse {
    let station = station.into_inner();
    let sensors: Vec<Sensor> = db
        .query("SELECT *, (SELECT * FROM sensor_value WHERE sensor.id = $parent.id ORDER BY server_timestamp DESC LIMIT 1) as values FROM sensor WHERE station.name = $station;;")
        .bind(("station", &station))
        .await
        .unwrap()
        .take(0)
        .unwrap();

    HttpResponse::Ok().json(sensors)
}

#[derive(Deserialize)]
struct SensorQuery {
    sensor: String,
    min: String,
    max: String,
}

#[post("/sensor/{sensor}/values")]
async fn get_sensor_values(
    json: web::Json<SensorQuery>,
    db: web::Data<Surreal<Client>>,
) -> HttpResponse {
    let thing = Thing::from(("sensor".to_string(), json.sensor.clone()));
    let sensor: Option<Sensor> = db
        .query(
            "SELECT *, (SELECT * FROM $sensor->hasValue->sensor_value WHERE server_timestamp < time::now() - <duration> $min AND server_timestamp > time::now() - <duration> $max ORDER BY server_timestamp ASC) AS values FROM $sensor",
        )
        .bind(("sensor", thing))
        .bind(("min", &json.min))
        .bind(("max", &json.max))
        .await
        .unwrap()
        .take(0)
        .unwrap();

    HttpResponse::Ok().json(sensor)
}
