use std::fs;

use diesel::r2d2::{ConnectionManager, Pool};
use rstest::*;
use serde::Deserialize;
use tracing::info;
use tracing::Level;
use uuid::Uuid;

use ledger::user::models::User;
use ledger::user::UserId;
use ledger::DbPool;

pub mod mock;

macro_rules! build_app {
    ($pool:ident, $user_id:expr) => {
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
                    .wrap(MockAuthentication { user_id: $user_id }),
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

#[derive(Deserialize)]
struct TestConfig {
    database_url: String,
}

pub struct TestUser {
    pub user_id: UserId,
    pool: DbPool,
}

impl TestUser {
    fn new(db_pool: &DbPool) -> TestUser {
        let user_id = "test-user-".to_owned() + &Uuid::new_v4().to_string();
        let user = User {
            id: user_id.to_string(),
            password_hash: ledger::auth::password::encode_password("pass".to_string()).unwrap(),
        };
        ledger::user::models::create_user(&db_pool, user).unwrap();
        info!(%user_id, "Created user");
        TestUser {
            user_id,
            pool: db_pool.clone(),
        }
    }
}

impl Drop for TestUser {
    fn drop(&mut self) {
        info!(%self.user_id, "Deleted user");
        ledger::user::models::delete_user(&self.pool, &self.user_id).unwrap();
    }
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

#[fixture]
pub fn test_user(database_pool: &DbPool) -> TestUser {
    TestUser::new(&database_pool)
}
