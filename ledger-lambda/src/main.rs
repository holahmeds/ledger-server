extern crate base64;

use actix_web::App;
use base64::Engine;
use lambda_web::{run_actix_on_lambda, LambdaError};
use ledger_lib::auth::jwt::JWTAuth;
use ledger_lib::config::Config;
use ledger_repo::sqlx_repo::create_repos;
use std::env;
use tracing::info;
use tracing::level_filters::LevelFilter;
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
        ledger_lib::tracing::create_opentelemetry_layer(SERVICE_NAME, &config.honeycomb_api_key)?;

    let subscriber = registry::Registry::default()
        .with(LevelFilter::INFO)
        .with(tracing_subscriber::fmt::Layer::default())
        .with(telemetry_layer);
    tracing::subscriber::set_global_default(subscriber).expect("set up subscriber");
    drop(tracing_guard);

    let base64_engine = base64::engine::general_purpose::STANDARD;
    let secret = base64_engine.decode(env::var("SECRET").expect("SECRET not set"))?;

    let (user_repo, transaction_repo, _transaction_template_repo) =
        create_repos(config.database_url, 1).await;

    let jwt_auth = JWTAuth::from_secret(secret);

    let factory = move || {
        App::new()
            .wrap(ledger_lib::tracing::create_middleware())
            .configure(ledger_lib::app_config_func(
                jwt_auth.clone(),
                transaction_repo.clone(),
                user_repo.clone(),
                config.signups_enabled,
            ))
    };
    run_actix_on_lambda(factory).await?;
    Ok(())
}
