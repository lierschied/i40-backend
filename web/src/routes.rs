use crate::middleware::authorization::JWTAuthorization;
use actix_web::{get, post, web, HttpResponse};
use common;
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

#[get("stations")]
async fn get_stations(db: web::Data<Surreal<Client>>) -> HttpResponse {
    let stations = common::Station::get_all(&db)
        .await
        .expect("Error retrieving stations");
    HttpResponse::Ok().json(stations)
}

#[get("/sensor/{sensor}")]
async fn get_sensor(sensor_id: web::Path<String>, db: web::Data<Surreal<Client>>) -> HttpResponse {
    let sensor_id = sensor_id.into_inner();
    let sensor: Option<common::Sensor> = common::Sensor::get(&db, sensor_id)
        .await
        .expect("Error retrieving sensor from database");

    HttpResponse::Ok().json(sensor)
}

#[get("/station/{station}/sensors")]
async fn get_sensors(
    station_id: web::Path<String>,
    db: web::Data<Surreal<Client>>,
) -> HttpResponse {
    let station_id = station_id.into_inner();
    let sensors: Vec<common::Sensor> = common::Sensor::get_by_station(&db, station_id)
        .await
        .expect("Error retrieving sensors by station");

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
