//! # web::config
//!
//! `web::config` is a module handling the configuration for the whole application
//! # Example
//!
//! ```text
//! secret: "my_little_secret"
//! restapi:
//!   base_url: "http://127.0.0.1"
//!   postfix: "mhubx-cc/module/juwi/action"
//!   port: 8001
//!   username: "user"
//!   password: "password"
//!
//! web:
//!   address: "0.0.0.0"
//!   port: 8080
//!
//! db:
//!   address: "127.0.0.1"
//!   port: 8000
//! ```

use serde::Deserialize;

/// struct containing the app state.
/// the secret is used for the JWTAuthorization middleware
/// restapi contains the MHubX rest API details
#[derive(Deserialize)]
pub struct AppState {
    pub secret: String,
    pub restapi: RestApi,
}

impl AppState {
    pub fn load() -> Self {
        let file = std::fs::read_to_string("web/config.yaml").expect("config.yaml not found!");

        serde_yaml::from_str(file.as_str()).expect("unable to parse config.yaml")
    }
}

/// main http-server config
#[derive(Deserialize)]
pub struct Config {
    pub web: ServerConfig,
    pub db: ServerConfig,
}

impl Config {
    pub fn load() -> Self {
        let file = std::fs::read_to_string("web/config.yaml").expect("config.yaml not found!");

        serde_yaml::from_str(file.as_str()).expect("unable to parse config.yaml")
    }
}

/// a struct to define a serverconfig with only address and port
#[derive(Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
}

/// contains the MHubX rest API details
#[derive(Deserialize)]
pub struct RestApi {
    pub base_url: String,
    pub port: String,
    pub postfix: String,
    pub username: String,
    pub password: String,
}
