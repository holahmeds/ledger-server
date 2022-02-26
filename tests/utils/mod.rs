use std::fs;
use std::sync::Once;

use diesel::r2d2::{ConnectionManager, Pool};
use rstest::*;
use serde::Deserialize;
use tracing::info;
use tracing::Level;

use ledger::DbPool;

static INIT_TESTS: Once = Once::new();
static mut DATABASE_POOL: Option<DbPool> = None;

#[derive(Deserialize)]
struct TestConfig {
    database_url: String,
}

fn setup() {
    INIT_TESTS.call_once(|| {
        tracing_subscriber::fmt().with_max_level(Level::INFO).init();
        info!("tracing initialized");

        let config = fs::read_to_string("config_test.toml").unwrap();
        let config: TestConfig = toml::from_str(config.as_str()).unwrap();

        let manager: ConnectionManager<diesel::PgConnection> =
            ConnectionManager::new(config.database_url);

        let pool = Pool::builder().build(manager).unwrap();
        unsafe {
            DATABASE_POOL = Some(pool);
        }
        info!("Database pool created");
    })
}

#[fixture]
pub fn database_pool() -> DbPool {
    setup();
    let pool = unsafe { DATABASE_POOL.as_ref().unwrap() };
    pool.clone()
}
