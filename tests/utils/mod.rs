use std::fs;
use std::sync::Arc;

use diesel::r2d2::{ConnectionManager, Pool};
use rstest::*;
use serde::Deserialize;
use tracing::info;
use tracing::Level;
use uuid::Uuid;

use ledger::transaction::models::DieselTransactionRepo;
use ledger::transaction::TransactionRepo;
use ledger::user::models::{DieselUserRepo, User};
use ledger::user::{UserId, UserRepo};
use ledger::DbPool;

pub mod mock;

macro_rules! build_app {
    ($transaction_repo:ident, $user_id:expr) => {
        App::new()
            .app_data(Data::new($transaction_repo))
            .wrap(ledger::tracing::create_middleware())
            .service(
                ledger::transaction::transaction_service()
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
    repo: Box<dyn UserRepo>,
}

impl TestUser {
    pub async fn new(user_repo: Box<dyn UserRepo>) -> TestUser {
        let user_id = "test-user-".to_owned() + &Uuid::new_v4().to_string();
        let user = User {
            id: user_id.to_string(),
            password_hash: ledger::auth::password::encode_password("pass".to_string()).unwrap(),
        };
        user_repo.create_user(user).await.unwrap();
        info!(%user_id, "Created user");
        TestUser {
            user_id,
            repo: user_repo,
        }
    }
}

impl Drop for TestUser {
    fn drop(&mut self) {
        futures::executor::block_on(self.repo.delete_user(&self.user_id)).unwrap();
        info!(%self.user_id, "Deleted user");
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
pub fn transaction_repo(database_pool: &DbPool) -> Arc<dyn TransactionRepo> {
    Arc::new(DieselTransactionRepo::new(database_pool.clone()))
}

#[fixture]
pub fn user_repo(database_pool: &DbPool) -> Box<dyn UserRepo> {
    Box::new(DieselUserRepo::new(database_pool.clone()))
}
