use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::header::AUTHORIZATION,
    web, Error,
};
use futures_util::future::LocalBoxFuture;

use crate::{auth::Claims, config::AppState};

pub struct JWTAuthorization;

impl<S, B> Transform<S, ServiceRequest> for JWTAuthorization
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JWTAuthorizationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JWTAuthorizationMiddleware { service }))
    }
}

pub struct JWTAuthorizationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JWTAuthorizationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let response = match req.headers().get(AUTHORIZATION) {
            Some(auth_header) => {
                let jwt = auth_header
                    .to_str()
                    .unwrap_or_default()
                    .split("Bearer")
                    .collect::<String>();

                let app_state = req.app_data::<web::Data<AppState>>().unwrap();
                match jsonwebtoken::decode::<Claims>(
                    jwt.as_str().trim(),
                    &jsonwebtoken::DecodingKey::from_secret(app_state.secret.as_str().as_ref()),
                    &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
                ) {
                    Ok(claims) => Ok(claims),
                    Err(err) => Err(ErrorUnauthorized(err)),
                }
            }
            None => Err(ErrorUnauthorized("Missing authorization header!")),
        };
        let fut = self.service.call(req);

        Box::pin(async move {
            match response {
                Ok(_) => {
                    let res = fut.await?;
                    Ok(res)
                }
                Err(err) => Err(err),
            }
        })
    }
}
