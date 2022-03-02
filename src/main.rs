extern crate jsonwebtoken;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate diesel_migrations;

use std::error::Error;
use std::fs;
use std::path::Path;

use actix_web::error::JsonPayloadError;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::HttpServer;
use actix_web::{web, App, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use serde::Deserialize;
use tracing::Level;

use ledger::auth;
use ledger::auth::JWTAuth;
use ledger::transaction;

#[derive(Deserialize)]
struct Config {
    database_url: String,
    secret: String,
}

embed_migrations!();

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    info!("tracing initialized");

    let config_path = get_config_file()?;
    let config = match fs::read_to_string(config_path) {
        Ok(s) => s,
        Err(e) => {
            error!("Unable to read config file: {}", e);
            return Err(e.into());
        }
    };
    let config: Config = toml::from_str(config.as_str())?;

    let manager: ConnectionManager<diesel::PgConnection> =
        ConnectionManager::new(config.database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Unable to build database pool");

    info!("Running migrations");
    let connection = pool.get()?;
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout())?;

    let jwt_auth = JWTAuth::from_base64_secret(config.secret).unwrap();
    info!("Token: {}", jwt_auth.create_token());

    HttpServer::new(move || {
        let state = Data::new(pool.clone());
        App::new()
            .app_data(jwt_auth.clone())
            .app_data(state)
            .wrap(HttpAuthentication::bearer(auth::request_validator))
            .wrap(Logger::default())
            .service(
                web::scope("/transactions")
                    .service(transaction::handlers::get_transaction)
                    .service(transaction::handlers::get_all_transactions)
                    .service(transaction::handlers::create_new_transaction)
                    .service(transaction::handlers::update_transaction)
                    .service(transaction::handlers::delete_transaction),
            )
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                match err {
                    JsonPayloadError::Deserialize(deserialize_err) => {
                        actix_web::error::InternalError::from_response(
                            "Unable to parse JSON",
                            HttpResponse::BadRequest()
                                .content_type("application/json")
                                .body(format!(r#"{{"error":"{}"}}"#, deserialize_err)),
                        )
                        .into()
                    }
                    _ => err.into(),
                }
            }))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await?;

    Ok(())
}

fn get_config_file() -> Result<&'static str, &'static str> {
    if Path::new("config.toml").exists() {
        return Ok("config.toml");
    }
    if cfg!(unix) {
        if Path::new("/etc/ledger/config.toml").exists() {
            return Ok("/etc/ledger/config.toml");
        }
    }

    Err("Config file not found")
}
