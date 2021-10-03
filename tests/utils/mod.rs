use std::env;
use std::sync::Once;

use actix_web::dev::{Body, ServiceResponse};
use actix_web::web::BytesMut;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use futures_util::StreamExt;
use rstest::*;
use serde::de::DeserializeOwned;
use tracing::info;
use tracing::Level;

use ledger::DbPool;

static INIT_TESTS: Once = Once::new();
static mut DATABASE_POOL: Option<DbPool> = None;

fn setup() {
    INIT_TESTS.call_once(|| {
        tracing_subscriber::fmt().with_max_level(Level::INFO).init();
        info!("tracing initialized");

        dotenv().ok();
        info!("dotenv initialized");

        let database_url = env::var("DATABASE_TEST_URL")
            .expect("DATABASE_TEST_URL not found in environment variables");
        let manager: ConnectionManager<diesel::PgConnection> = ConnectionManager::new(database_url);

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

pub async fn map_body<T>(input: &mut ServiceResponse<Body>) -> T
    where
        T: DeserializeOwned,
{
    let mut body = input.take_body();
    let mut bytes = BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }

    info!("{:?}", bytes);

    let result: T = serde_json::from_slice(&bytes).unwrap();
    result
}
