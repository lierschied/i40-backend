//! # common
//!
//! `common` is a collection of utilities to share across the different services
//!

use chrono::Utc;
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{Datetime, Id, Thing, Value},
    Surreal,
};

type DB = Surreal<Client>;

/// Station
///
/// # Example
///
/// ```
/// let station = Station::new("palettenlager".to_owned());
///
/// let answer = Station {
///     id: Thing::from(("station", "palettenlager")),
///     name: "palettenlager".to_owned(),
/// };
///
/// assert_eq!(station, answer);
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct Station {
    id: Thing,
    name: String,
    sensors: Option<Vec<Sensor>>,
}

impl Station {
    /// Returns a new Station
    ///
    /// # Arguments
    /// name: String -> needs to be unique
    ///
    /// ```
    /// let station = Station::new("palettenlager".to_owned());
    ///
    /// let answer = Station {
    ///     id: Thing::from(("station", "palettenlager")),
    ///     name: "palettenlager".to_owned(),
    /// };
    ///
    /// assert_eq!(station, answer);
    /// ```
    pub fn new(name: String) -> Self {
        Station {
            id: Thing::from(("station", name.as_str())),
            name,
            sensors: None,
        }
    }

    /// Returns a vector of all available stations
    pub async fn get_all(db: &DB) -> Result<Vec<Self>, surrealdb::Error> {
        db.select("station").await
    }

    /// Returns a station by id or None if the id does not exist
    pub async fn get(db: &DB, id: String) -> Result<Option<Self>, surrealdb::Error> {
        db.select(Thing::from(("station", id.as_str()))).await
    }

    pub fn get_id(&self) -> &Thing {
        &self.id
    }
}

/// Sensor
///
/// # Example
///
/// ```
/// let sensor = Sensor::new("dosenfuellstand", Thing::from(("station", "palettenlager")));
///
/// let answer = Sensor {
///     id: Thing::from(("sensor", "dosenfuellstand")),
///     station: Thing::from(("station", "palettenlager")),
///     display_name: "dosenfuellstand".to_owned(),
///     values: None,
/// };
///
/// assert_eq!(sensor, answer);
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct Sensor {
    id: Thing,
    station: Thing,
    display_name: String,
    values: Option<Vec<SensorValue>>,
}

impl Sensor {
    /// Creates a new Sensor struct
    pub fn new(name: String, station: Thing) -> Self {
        Sensor {
            id: Thing::from(("sensor", name.as_str())),
            station,
            display_name: name.into(),
            values: None,
        }
    }

    /// Creates a new Sensor struct an saves it to the database
    pub async fn create(
        db: &DB,
        name: String,
        station: Thing,
    ) -> Result<Option<Self>, surrealdb::Error> {
        db.create("sensor").content(Self::new(name, station)).await
    }

    /// Retrive a single sensor, without values by its id
    pub async fn get(db: &DB, id: String) -> Result<Option<Self>, surrealdb::Error> {
        db.select(Thing::from(("sensor", id.as_str()))).await
    }

    /// Retrive a list of sensors, without the latest value by station id
    pub async fn get_by_station(db: &DB, id: String) -> Result<Vec<Self>, surrealdb::Error> {
        db.query("SELECT *, (SELECT * FROM sensor_value:[$parent.id, NONE]..[$parent.id, time::now()] ORDER BY server_timestamp ASC LIMIT 1 ) AS values FROM sensor WHERE station = $station;")
            .bind(("station", Thing::from(("station", id.as_str()))))
            .await
            .expect("Error during query execution")
            .take(0)
    }

    /// Returns a Sensor and all its values
    ///
    /// # Example
    ///
    /// ```
    /// let sensor: Option<Sensor> = Sensor::get_values(&db, "dosenfuellstand").await.unwrap();
    /// ```
    pub async fn get_with_values(db: &DB, id: String) -> Result<Option<Self>, surrealdb::Error> {
        db.query("SELECT *, (SELECT * FROM sensor_value:[$sensor, NONE]..) AS values FROM $sensor")
            .bind(("sensor", Thing::from(("sensor".to_string(), id))))
            .await
            .unwrap()
            .take(0)
    }

    /// Returns a Sensor and its values within a time period
    ///
    /// # Example
    ///
    /// ```
    /// let time_period = TimeRange::new(chrono::Duration::Hour(5), Utc::now());
    /// let sensor: Option<Sensor> = Sensor::get_values_within_timeperiod(&db, "dosenfuellstand", time_period).await.unwrap();
    /// ```
    pub async fn get_values_within_timeperiod(
        db: &DB,
        id: String,
        time_period: TimePeriod,
    ) -> Result<Option<Self>, surrealdb::Error> {
        db.query("SELECT *, (SELECT * FROM sensor_value:[$sensor, $from]..[$sensor, $to]) AS values FROM $sensor")
            .bind(("sensor", Thing::from(("sensor".to_string(), id))))
            .bind(("from", time_period.from))
            .bind(("to", time_period.to))
            .await
            .expect("Error during query")
            .take(0)
    }

    /// Returns the id of this [`Sensor`].
    pub fn get_id(&self) -> &Thing {
        &self.id
    }
}

/// A time period within which to query data
///
/// # Example
/// A period from before 5 hours to now
/// now = 15:00 => from = 10:00
///
/// ```
/// let from = chrono::Duration::hours(5);
/// let to = Some(chrono::Utc::now());
/// let time_period = TimeRange::between(from, to);
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct TimePeriod {
    from: Option<Datetime>,
    to: Option<Datetime>,
}

impl TimePeriod {
    /// Create a time period  
    /// If a chrono::duration is passed, it will be subtracted from the current time
    ///
    /// # Example
    /// A period from five hours ago to now
    /// now = 15:00 => from = 10:00
    ///
    /// ```
    /// let from = chrono::Duration::hours(5);
    /// let to = Some(chrono::Utc::now());
    /// let time_period = TimePeriod::between(from, to);
    /// ```
    pub fn between(from: impl ToDatetime, to: impl ToDatetime) -> Self {
        Self {
            from: from.to_datetime(),
            to: to.to_datetime(),
        }
    }

    /// Create a time period between from till now
    ///
    /// # Example
    /// A period from five minutes ago to now
    ///
    /// ```
    /// let from = Some(chrono::Utc::now() - chrono::Duration::Minutes(5))
    /// let time_period = TimeRange::from(from);
    /// ```
    pub fn from(from: impl ToDatetime) -> Self {
        Self {
            from: from.to_datetime(),
            to: Some(Datetime(Utc::now())),
        }
    }
}

/// A trait to convert a given time into A surrealdb parsable Datetime
pub trait ToDatetime {
    fn to_datetime(self) -> Option<Datetime>;
}

impl ToDatetime for Option<chrono::DateTime<Utc>> {
    fn to_datetime(self) -> Option<Datetime> {
        match self {
            Some(v) => Some(Datetime(v)),
            None => None,
        }
    }
}

impl ToDatetime for chrono::Duration {
    fn to_datetime(self) -> Option<Datetime> {
        Some(Datetime(chrono::Utc::now() - self))
    }
}

impl ToDatetime for chrono::DateTime<Utc> {
    fn to_datetime(self) -> Option<Datetime> {
        Some(Datetime(self))
    }
}

/// A struct to repesent a single value at a given time from a sensor
///
/// # Example
///
/// ```
/// let sensor = Thing::from(("sensor", "doesnfuellstand"));
/// let value = SensorValue::new("12", sensor);
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct SensorValue {
    id: Thing,
    sensor: Thing,
    value: String,
    server_timestamp: Datetime,
}

impl SensorValue {
    /// Creates a new sensor value struct
    pub fn new(value: String, sensor: Thing) -> Self {
        let server_timestamp = Datetime(Utc::now());
        SensorValue {
            id: Thing::from((
                "sensor_value".to_owned(),
                Id::from(vec![
                    Value::Thing(sensor.clone()),
                    Value::Datetime(server_timestamp.clone()),
                ]),
            )),
            sensor,
            value,
            server_timestamp,
        }
    }

    /// Creates a new sensor_value struct and saves it to the database
    pub async fn create(
        db: &DB,
        value: String,
        sensor: Thing,
    ) -> Result<Option<Self>, surrealdb::Error> {
        db.create("sensor_value")
            .content(Self::new(value, sensor))
            .await
    }
}
