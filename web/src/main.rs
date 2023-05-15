use actix_cors::Cors;
use actix_files::NamedFile;
use std::path::Path;
mod api;
mod app;
mod auth;
mod config;
mod middleware;
mod routes;

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};

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
    .unwrap();

    db.use_ns("main").use_db("main").await.unwrap();

    let app_state = web::Data::new(app_state);
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(app_state.clone())
            .wrap(cors)
            .service(app::user::sign_in)
            .service(get_person)
            .service(create_person)
            .service(get_persons)
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

trait DbData<T> {
    fn get(id: String, db: &Surreal<Client>) -> Option<T>;
    fn all() -> Vec<T>;
    fn create(self: Self) -> T;
    fn update(self: Self) -> T;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Person {
    id: Option<Thing>,
    name: String,
}

#[derive(serde::Deserialize, Debug)]
struct PersonInfo {
    name: String,
}

#[post("/person")]
async fn create_person(
    db: web::Data<Surreal<Client>>,
    info: web::Json<PersonInfo>,
) -> HttpResponse {
    let r: Person = db
        .create("person")
        .content(Person {
            id: None,
            name: info.name.clone(),
        })
        .await
        .unwrap();
    dbg!(&r);
    HttpResponse::Ok().json(r)
}

#[get("/persons")]
async fn get_persons(db: web::Data<Surreal<Client>>) -> HttpResponse {
    let persons: Vec<Person> = db.select("person").await.unwrap();
    HttpResponse::Ok().json(persons)
}

#[get("/person/{id}")]
async fn get_person(db: web::Data<Surreal<Client>>, info: web::Path<String>) -> HttpResponse {
    let id = info.into_inner();
    let person: Option<Person> = db.select(("person", id)).await.unwrap();
    HttpResponse::Ok().json(person)
}
