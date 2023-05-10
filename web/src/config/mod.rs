use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppState {
    pub secret: String,
    pub restapi: RestApi,
}

#[derive(Deserialize)]
pub struct RestApi {
    pub base_url: String,
    pub port: String,
    pub postfix: String,
    pub username: String,
    pub password: String,
}

impl AppState {
    pub fn load() -> Self {
        let file = std::fs::read_to_string("web/config.yaml").expect("config.yaml not found!");

        serde_yaml::from_str(file.as_str()).expect("unable to parse config.yaml")
    }
}
