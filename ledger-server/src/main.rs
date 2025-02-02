extern crate jsonwebtoken;
#[macro_use]
extern crate tracing;
extern crate serde_json;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::Context;
use rand::Rng;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry;

use ledger_lib::auth::jwt::JWTAuth;
use ledger_lib::config::Config;
use ledger_repo::sqlx_repo::SQLxRepo;
use ledger_repo::transaction_repo::TransactionRepo;
use ledger_repo::user_repo::UserRepo;
use ledger_repo::HealthCheck;
use ledger_repo::transaction_template_repo::TransactionTemplateRepo;

const SERVICE_NAME: &str = "ledger-server";

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let subscriber = registry::Registry::default()
        .with(LevelFilter::INFO)
        .with(tracing_subscriber::fmt::Layer::default());
    let tracing_guard = tracing::subscriber::set_default(subscriber);
    info!("tracing initialized");

    let config_path = get_config_file()?;
    let config: Config = Config::from_file(config_path)?;

    let telemetry_layer =
        ledger_lib::tracing::create_opentelemetry_layer(SERVICE_NAME, &config.honeycomb_api_key)?;

    let subscriber = registry::Registry::default()
        .with(LevelFilter::INFO)
        .with(tracing_subscriber::fmt::Layer::default())
        .with(telemetry_layer);
    tracing::subscriber::set_global_default(subscriber).expect("set up subscriber");
    drop(tracing_guard);

    let repo = SQLxRepo::new(config.database_url, 10).await?;
    let transaction_repo: Arc<dyn TransactionRepo> = Arc::new(repo.clone());
    let template_repo: Arc<dyn TransactionTemplateRepo> = Arc::new(repo.clone());
    let user_repo: Arc<dyn UserRepo> = Arc::new(repo.clone());
    let repo_health: Arc<dyn HealthCheck> = Arc::new(repo);

    let secret = get_secret()?;
    let jwt_auth = JWTAuth::from_secret(secret);

    let mut server = HttpServer::new(move || {
        let cors = Cors::permissive(); // We do authentication using the Authorization header, so don't need CORS
        App::new()
            .wrap(cors)
            .wrap(ledger_lib::tracing::create_middleware())
            .configure(ledger_lib::app_config_func(
                jwt_auth.clone(),
                user_repo.clone(),
                transaction_repo.clone(),
                template_repo.clone(),
                config.signups_enabled,
            ))
            .configure(ledger_lib::health_check_config_func(repo_health.clone()))
    });
    server = match config.ssl {
        None => {
            warn!("Using http");
            server.bind("[::]:8000")?
        }
        Some(ssl_config) => {
            info!("Using https");

            let config = ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth();

            let mut cert_file = BufReader::new(
                File::open(ssl_config.certificate_chain_file)
                    .context("Error opening certificate chain file")?,
            );
            let mut key_file = BufReader::new(
                File::open(ssl_config.private_key_file)
                    .context("Error opening private key file")?,
            );

            let cert_chain = certs(&mut cert_file)
                .context("Unable to read certificate chain file")?
                .into_iter()
                .map(Certificate)
                .collect();
            let mut keys: Vec<PrivateKey> = pkcs8_private_keys(&mut key_file)
                .context("Unable to read private key file")?
                .into_iter()
                .map(PrivateKey)
                .collect();

            if keys.is_empty() {
                error!("No private key found in file");
                std::process::exit(1);
            }

            let config = config.with_single_cert(cert_chain, keys.remove(0))?;

            server.bind_rustls("[::]:8000", config)?
        }
    };
    server.run().await?;

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
