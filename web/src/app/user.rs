//! # web::user
//!
//! `web::user` is a module to handle all interactions for a user
//!
//! # Example
//!

use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};

use crate::config::AppState;

/// A struct containing all necessary user data
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct User {
    id: Option<Thing>,
    email: String,
    name: String,
    jwt: Option<String>,
    icon: Option<String>,
}

/// helper struct to deserialize the login form
#[derive(Deserialize)]
pub struct SignInFormData {
    email: String,
    password: String,
}

/// route to login a existing user
#[post("/signin")]
pub async fn sign_in(
    app_state: web::Data<AppState>,
    db: web::Data<Surreal<Client>>,
    form: web::Form<SignInFormData>,
) -> HttpResponse {
    let user: Option<User> = db.query("SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(password, $password)")
    .bind(("email", form.email.clone()))
    .bind(("password", form.password.clone()))
    .await
    .unwrap()
    .take(0)
    .unwrap();

    if user.is_none() {
        return HttpResponse::NotFound().body("Wrong email or password");
    }

    let mut user = user.unwrap();
    user.jwt = Some(crate::auth::generate_token(
        app_state.secret.clone(),
        user.email.clone(),
        1,
    ));
    HttpResponse::Ok().json(user)
}

/// helper struct to deserialize the User update form
#[derive(Deserialize)]
pub struct UserFormData {
    user: String,
    name: String,
    icon: String,
}

/// endpoint to update the user data
#[post("/user/update")]
pub async fn update_user(
    app_state: web::Data<AppState>,
    db: web::Data<Surreal<Client>>,
    form: web::Form<UserFormData>,
) -> HttpResponse {
    let user: Option<User> = db
        .query("UPDATE $user SET name = $name, icon = $icon")
        .bind(("user", &form.user))
        .bind(("name", &form.name))
        .bind(("icon", &form.icon))
        .await
        .unwrap()
        .take(0)
        .unwrap();

    if user.is_none() {
        return HttpResponse::NotFound().body("Wrong email or password");
    }

    let mut user = user.unwrap();
    user.jwt = Some(crate::auth::generate_token(
        app_state.secret.clone(),
        user.email.clone(),
        1,
    ));
    HttpResponse::Ok().json(user)
}

/// endpoint to register a new user
/// !!!dummy implementation
#[post("/signup")]
pub async fn sign_up(db: web::Data<Surreal<Client>>) -> HttpResponse {
    let r = db
    .query("CREATE user SET email = $email, name = 'Hugo', password = crypto::argon2::generate($password)")
    .bind(("email", "test@mail.de"))
    .bind(("password", "1234"))
    .await.unwrap();
    dbg!(r);
    HttpResponse::Ok().body("1")
}
