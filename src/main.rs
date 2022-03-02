extern crate jsonwebtoken;
#[macro_use]
extern crate tracing;

use std::fs;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    info!("tracing initialized");

    let config = match fs::read_to_string("config.toml") {
        Ok(s) => s,
        Err(e) => {
            error!("Unable to read config file: {}", e);
            return Err(e);
        }
    };
    let config: Config = toml::from_str(config.as_str())?;

    let manager: ConnectionManager<diesel::PgConnection> =
        ConnectionManager::new(config.database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Unable to build database pool");

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
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
