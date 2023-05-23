//! # web::routes
//!
//! `web::routes` is the central module for defining the api routes
//!

use crate::middleware::authorization::JWTAuthorization;
use actix_web::{get, post, web, HttpResponse};
use common;
use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, Surreal};

/// defining all api routes behind a JWTAuthorization middleware
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

/// endpoint to retrieve all stations
#[get("stations")]
async fn get_stations(db: web::Data<Surreal<Client>>) -> HttpResponse {
    let stations = common::Station::get_all(&db)
        .await
        .expect("Error retrieving stations");
    HttpResponse::Ok().json(stations)
}

/// endpoint to retrieve a sensor
#[get("/sensor/{sensor}")]
async fn get_sensor(sensor_id: web::Path<String>, db: web::Data<Surreal<Client>>) -> HttpResponse {
    let sensor_id = sensor_id.into_inner();
    let sensor: Option<common::Sensor> = common::Sensor::get(&db, sensor_id)
        .await
        .expect("Error retrieving sensor from database");

    HttpResponse::Ok().json(sensor)
}

/// endpoint to retrive all sensors with the latest value for a given station
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

/// helper struct to Deserialize the payload
#[derive(Deserialize)]
struct SensorQuery {
    sensor: String,
    to: String,
    from: String,
}

/// endpoint to retrive all values for a given sensor
#[post("/sensor/{sensor}/values")]
async fn get_sensor_values(
    json: web::Json<SensorQuery>,
    db: web::Data<Surreal<Client>>,
) -> HttpResponse {
    let to = chrono::Duration::minutes(
        json.to
            .parse::<i64>()
            .expect("unable to convert min to int"),
    );
    let from = chrono::Duration::minutes(
        json.from
            .parse::<i64>()
            .expect("unable to convert max to int"),
    );

    let sensor = common::Sensor::get_values_within_timeperiod(
        &db,
        json.sensor.clone(),
        common::TimePeriod::between(from, to),
    )
    .await
    .expect("unable to retrive sensor");

    HttpResponse::Ok().json(sensor)
}
