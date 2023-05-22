use actix_cors::Cors;
use actix_files::NamedFile;
use std::path::Path;
mod api;
mod app;
mod auth;
mod config;
mod middleware;
mod routes;

use actix_web::{get, web, App, HttpServer};
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = crate::config::AppState::load();

    let db = Surreal::new::<Ws>("127.0.0.1:8000")
        .await
        .expect("Unable to connect to database");

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .expect("Unable to sigin to the database");

    db.use_ns("main")
        .use_db("main")
        .await
        .expect("Either namespace or database main does not exist");

    let app_state = web::Data::new(app_state);
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(app_state.clone())
            .wrap(cors)
            .service(app::user::sign_in)
            .configure(routes::config)
            .service(single_page_app)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("{filename:.*}")]
async fn single_page_app(file_name: web::Path<String>) -> Result<NamedFile, actix_web::Error> {
    let mut file_path = format!("../frontend/dist/{}", file_name.into_inner());
    if !Path::new(&file_path).is_file() {
        file_path = "../frontend/dist/index.html".to_string();
    }
    Ok(NamedFile::open(file_path)?)
}
