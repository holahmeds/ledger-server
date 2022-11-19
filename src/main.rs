extern crate jsonwebtoken;
#[macro_use]
extern crate tracing;
extern crate serde_json;

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use actix_web::error::JsonPayloadError;
use actix_web::web::Data;
use actix_web::{web, App};
use actix_web::{HttpResponse, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use rand::Rng;
use serde::Deserialize;
use tracing::Level;

use ledger::auth::jwt::JWTAuth;
use ledger::transaction;
use ledger::{auth, user};

#[derive(Deserialize)]
struct Config {
    database_url: String,
    signups_enabled: bool,
}

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

    let (transaction_repo, user_repo) =
        ledger_repo::sqlx_repo::create_repos(config.database_url, 10).await;

    let secret = get_secret()?;
    let jwt_auth = JWTAuth::from_secret(secret);
    let bearer_auth_middleware = HttpAuthentication::bearer(auth::credentials_validator);

    HttpServer::new(move || {
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
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await?;

    Ok(())
}

fn get_config_file() -> Result<PathBuf, &'static str> {
    let config_current_dir = PathBuf::from("config.toml");
    if config_current_dir.exists() {
        return Ok(config_current_dir);
    }
    if let Ok(config_env) = std::env::var("CONFIGURATION_DIRECTORY") {
        let config_path = PathBuf::from(config_env).join("config.toml");
        if config_path.exists() {
            return Ok(config_path);
        }
    }

    Err("Config file not found")
}

fn get_state_dir() -> PathBuf {
    if let Ok(state_env) = std::env::var("STATE_DIRECTORY") {
        return PathBuf::from(state_env);
    }

    PathBuf::from("data")
}

/// Gets the secret from file. If the file does not exist it will generate a new secret and save it
/// to the file
fn get_secret() -> Result<Vec<u8>, Box<dyn Error>> {
    let state_dir = get_state_dir();
    let secret_file = state_dir.join("secret");
    if secret_file.exists() {
        Ok(fs::read(secret_file)?)
    } else {
        let mut rng = rand::thread_rng();
        let mut secret: [u8; 128] = [0; 128];
        rng.fill(&mut secret);

        fs::create_dir_all(state_dir)?;
        fs::write(secret_file, secret)?;

        Ok(secret.to_vec())
    }
}
