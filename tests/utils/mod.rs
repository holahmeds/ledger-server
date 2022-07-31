use std::fs;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::result::DatabaseErrorKind;
use diesel::result::Error::DatabaseError;
use rstest::*;
use serde::Deserialize;
use tracing::instrument;
use tracing::Level;
use tracing::{info, warn};

use ledger::user::models::User;
use ledger::DbPool;

pub mod mock;

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
    let db_conn = database_pool.get().unwrap();
    let result = ledger::user::models::create_user(&db_conn, user);
    if let Err(DatabaseError(DatabaseErrorKind::UniqueViolation, _)) = result {
        warn!("User already existed");
    }
}

#[instrument(skip(database_pool))]
pub fn delete_user(database_pool: &DbPool, user_id: &str) {
    ledger::user::models::delete_user(&database_pool.get().unwrap(), user_id).unwrap();
}
