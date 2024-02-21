#[macro_use]
extern crate actix_web;

use crate::auth::jwt::JWTAuth;
use ::tracing::error;
use actix_web::error::JsonPayloadError;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use ledger_repo::transaction_repo::TransactionRepo;
use ledger_repo::user_repo::UserRepo;
use ledger_repo::HealthCheck;
use std::sync::Arc;

pub mod auth;
pub mod config;
mod error;
pub mod tracing;
pub mod transaction;
pub mod user;

pub fn app_config_func(
    jwt_auth: JWTAuth,
    transaction_repo: Arc<dyn TransactionRepo>,
    user_repo: Arc<dyn UserRepo>,
    signups_enabled: bool,
) -> impl FnOnce(&mut web::ServiceConfig) {
    let bearer_auth_middleware = HttpAuthentication::bearer(auth::credentials_validator);

    move |cfg| {
        cfg.app_data(jwt_auth)
            .app_data(Data::new(transaction_repo.clone()))
            .app_data(Data::new(user_repo.clone()))
            .service(transaction::transaction_service().wrap(bearer_auth_middleware.clone()))
            .service(user::user_service().wrap(bearer_auth_middleware.clone()))
            .service(auth::auth_service(signups_enabled))
            .app_data(web::JsonConfig::default().error_handler(|err, req| {
                error!(req_path = req.path(), %err);
                match err {
                    JsonPayloadError::Deserialize(deserialize_err) => {
                        let error_body = serde_json::json!({
                            "error": "Unable to parse JSON payload",
                            "detail": format!("{}", deserialize_err),
                        });
                        actix_web::error::InternalError::from_response(
                            deserialize_err,
                            HttpResponse::BadRequest()
                                .content_type("application/json")
                                .body(error_body.to_string()),
                        )
                            .into()
                    }
                    _ => err.into(),
                }
            }));
    }
}

#[get("/health")]
async fn health_check(repo_health: Data<Arc<dyn HealthCheck>>) -> HttpResponse {
    if repo_health.check().await {
        HttpResponse::Ok()
    } else {
        HttpResponse::InternalServerError()
    }
        .finish()
}

pub fn health_check_config_func(
    repo_health: Arc<dyn HealthCheck>,
) -> impl FnOnce(&mut web::ServiceConfig) {
    |cfg: &mut web::ServiceConfig| {
        cfg.app_data(Data::new(repo_health)).service(health_check);
    }
}
