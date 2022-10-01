use std::fs;

use diesel::r2d2::{ConnectionManager, Pool};
use rstest::*;
use serde::Deserialize;
use tracing::instrument;
use tracing::Level;
use tracing::{info, warn};

use ledger::user::models::{User, UserRepoError};
use ledger::DbPool;

pub mod mock;

macro_rules! build_app {
    ($pool:ident, $user_id:ident) => {
        App::new()
            .app_data(Data::new($pool.clone()))
            .wrap(ledger::tracing::create_middleware())
            .service(
                web::scope("/transactions")
                    .service(handlers::get_transaction)
                    .service(handlers::get_transactions)
                    .service(handlers::create_new_transaction)
                    .service(handlers::update_transaction)
                    .service(handlers::delete_transaction)
                    .wrap(MockAuthentication {
                        user_id: $user_id.clone(),
                    }),
            )
    };
}

macro_rules! create_transaction {
    (&$service:ident, $new_transaction:ident) => {{
        let request = TestRequest::post()
            .uri("/transactions")
            .set_json(&$new_transaction)
            .to_request();
        let response = test::call_service(&$service, request).await;
        assert!(
            response.status().is_success(),
            "Got {} response when creating transaction",
            response.status()
        );
        test::read_body_json(response).await
    }};
}

macro_rules! delete_transaction {
    (&$service:ident, $transaction_id:expr) => {{
        let delete_request = TestRequest::delete()
            .uri(format!("/transactions/{}", $transaction_id).as_str())
            .to_request();
        let response = test::call_service(&$service, delete_request).await;
        assert!(
            response.status().is_success(),
            "Got {} response when deleting transaction",
            response.status()
        )
    }};
}

#[derive(Deserialize)]
struct TestConfig {
    database_url: String,
}

#[fixture]
#[once]
pub fn database_pool() -> DbPool {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    info!("tracing initialized");

    let config = fs::read_to_string("config_test.toml").unwrap();
    let config: TestConfig = toml::from_str(config.as_str()).unwrap();

    let manager: ConnectionManager<diesel::PgConnection> =
        ConnectionManager::new(config.database_url);

    let pool = Pool::builder().build(manager).unwrap();
    info!("Database pool created");

    pool
}

#[instrument(skip(database_pool))]
pub fn create_user(database_pool: &DbPool, user_id: &str) {
    let user = User {
        id: user_id.to_string(),
        password_hash: ledger::auth::password::encode_password("pass".to_string()).unwrap(),
    };
    let result = ledger::user::models::create_user(&database_pool, user);
    if let Err(UserRepoError::UserNotFound(_)) = result {
        warn!("User already existed");
    }
}

#[instrument(skip(database_pool))]
pub fn delete_user(database_pool: &DbPool, user_id: &str) {
    ledger::user::models::delete_user(&database_pool, user_id).unwrap();
}
