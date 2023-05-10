use actix_web::{get, web};
use serde::{Deserialize, Serialize};

use crate::config::AppState;

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
    code: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: usize,
    pub name: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn generate_token(secret: String, username: String, user_id: i32) -> String {
    let sub: usize = user_id as usize;
    let exp: usize = (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as usize;
    let iat: usize = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub,
        name: username.clone(),
        iat,
        exp,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_str().as_ref()),
    )
    .unwrap()
}

#[get("/token")]
async fn decode(
    req: actix_web::HttpRequest,
    app_state: web::Data<AppState>,
) -> actix_web::HttpResponse {
    let auth_header = req.headers().get(actix_web::http::header::AUTHORIZATION);
    let jwt = match auth_header {
        Some(auth) => auth.to_str().unwrap(),
        _ => {
            return actix_web::HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Missing Authorization header!".to_string(),
                code: 401,
            });
        }
    };

    match jsonwebtoken::decode::<Claims>(
        &jwt.replace("Bearer ", ""),
        &jsonwebtoken::DecodingKey::from_secret(app_state.secret.as_str().as_ref()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    ) {
        Ok(token_data) => actix_web::HttpResponse::Ok().json(token_data.claims),
        Err(err) => actix_web::HttpResponse::Unauthorized().json(ErrorResponse {
            error: err.to_string(),
            code: 401,
        }),
    }
}

