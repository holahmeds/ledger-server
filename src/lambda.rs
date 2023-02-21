extern crate base64;

use actix_web::error::JsonPayloadError;
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use base64::Engine;
use lambda_web::{run_actix_on_lambda, LambdaError};
use ledger::auth::jwt::JWTAuth;
use ledger::config::Config;
use ledger::{auth, transaction, user};
use ledger_repo::sqlx_repo::create_repos;
use std::env;
use tracing::level_filters::LevelFilter;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry;

const SERVICE_NAME: &str = "ledger-lambda";

#[actix_web::main]
async fn main() -> Result<(), LambdaError> {
    let subscriber = registry::Registry::default()
        .with(LevelFilter::INFO)
        .with(tracing_subscriber::fmt::Layer::default());
    let tracing_guard = tracing::subscriber::set_default(subscriber);
    info!("tracing initialized");

    let config = Config::from_env().expect("config set up");

    let telemetry_layer =
        ledger::tracing::create_opentelemetry_layer(SERVICE_NAME, config.honeycomb)?;

    let subscriber = registry::Registry::default()
        .with(LevelFilter::INFO)
        .with(tracing_subscriber::fmt::Layer::default())
        .with(telemetry_layer);
    tracing::subscriber::set_global_default(subscriber).expect("set up subscriber");
    drop(tracing_guard);

    let base64_engine = base64::engine::general_purpose::STANDARD;
    let secret = base64_engine.decode(env::var("SECRET").expect("SECRET not set"))?;

    let (transaction_repo, user_repo) = create_repos(config.database_url, 1).await;

    let jwt_auth = JWTAuth::from_secret(secret);
    let bearer_auth_middleware = HttpAuthentication::bearer(auth::credentials_validator);

    let factory = move || {
        App::new()
            .app_data(jwt_auth.clone())
            .app_data(Data::new(transaction_repo.clone()))
            .app_data(Data::new(user_repo.clone()))
            .wrap(ledger::tracing::create_middleware())
            .service(transaction::transaction_service().wrap(bearer_auth_middleware.clone()))
            .service(user::user_service().wrap(bearer_auth_middleware.clone()))
            .service(auth::auth_service(config.signups_enabled))
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
            }))
    };
    run_actix_on_lambda(factory).await?;
    Ok(())
}
