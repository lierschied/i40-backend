use actix_web::{get, web, HttpResponse};

use crate::config::{AppState, RestApi};

//constructs the url endpoint for the MHubx RestAPI
fn get_endpoint(restapi: &RestApi, path: impl std::fmt::Display) -> String {
    format!(
        "{}:{}/{}{}",
        restapi.base_url, restapi.port, restapi.postfix, path
    )
}

//construcss the request and adds basic auth headers
//path is supposed to be the path after the base url
//e.g. "?page=Logic.Interface" becomes "http://example.com:8000/base/uri?page=Logic.Interface"
fn get(restapi: &RestApi, path: impl std::fmt::Display) -> reqwest::RequestBuilder {
    let client = reqwest::Client::new();
    let url = get_endpoint(&restapi, path);
    println!("{}", &url);
    client
        .get(url)
        .basic_auth(&restapi.username, Some(&restapi.password))
}

//returns the measurments for all systems from the MHubx RestAPI
#[get("/measurements")]
async fn get_measurments(state: web::Data<AppState>) -> HttpResponse {
    //system cps1 or * for all systems
    let resp = get(
        &state.restapi,
        "?page=Logic.Interface&name=getMeasurement&source=system&system_id=*&msm_id=*",
    )
    .send()
    .await
    .unwrap();
    dbg!(&resp);
    let json: serde_json::value::Value = resp.json().await.unwrap();
    HttpResponse::Ok().json(json)
}

//returns all alarms for all systems from the MHubx RestAPI
#[get("/alarms")]
async fn get_alarms(state: web::Data<AppState>) -> HttpResponse {
    let resp = get(
        &state.restapi,
        "?page=Logic.Interface&name=getAlarms&system_id=cps1",
    )
    .send()
    .await
    .unwrap();

    let json: serde_json::value::Value = resp.json().await.unwrap();
    HttpResponse::Ok().json(json)
}
