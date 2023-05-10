use crate::middleware::authorization::JWTAuthorization;
use actix_web::{get, web, HttpResponse};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};

pub fn config(app: &mut web::ServiceConfig) {
    app.service(
        web::scope("/api/v1")
            .wrap(JWTAuthorization)
            .service(crate::auth::decode)
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
    value: String,
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
        .query("SELECT *, (SELECT value FROM sensor_value WHERE sensor.station.name = $name) as values FROM sensor WHERE station.name = $name AND values.sensor.id = sensor.id")
        .bind(("name", &station))
        .await
        .unwrap()
        .take(0)
        .unwrap();

    HttpResponse::Ok().json(sensors)
}

#[get("/sensor/{sensor}/values")]
async fn get_sensor_values(
    sensor: web::Path<String>,
    db: web::Data<Surreal<Client>>,
) -> HttpResponse {
    let sensors: Vec<SensorValue> = db
        .query("SELECT * FROM sensor_value WHERE sensor.display_name = $sensor")
        .bind(("sensor", sensor.into_inner()))
        .await
        .unwrap()
        .take(0)
        .unwrap();
    HttpResponse::Ok().json(sensors)
}
