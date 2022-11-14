use std::fs;
use std::sync::Arc;

use rstest::*;
use serde::Deserialize;
use tracing::info;
use tracing::Level;
use uuid::Uuid;

use ledger::repo::sqlx::create_repos;
use ledger::repo::transaction_repo::TransactionRepo;
use ledger::repo::user_repo::User;
use ledger::repo::user_repo::UserRepo;
use ledger::user::UserId;

pub mod mock;

macro_rules! build_app {
    ($transaction_repo:ident, $user_id:expr) => {{
        let app = App::new()
            .app_data(Data::new($transaction_repo))
            .wrap(ledger::tracing::create_middleware())
            .service(
                ledger::transaction::transaction_service()
                    .wrap(MockAuthentication { user_id: $user_id }),
            );
        tracing::info!("Built app");
        app
    }};
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
    repo: Arc<dyn UserRepo>,
}

impl TestUser {
    pub async fn new(user_repo: Arc<dyn UserRepo>) -> TestUser {
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

    pub async fn delete(&self) {
        self.repo.delete_user(&self.user_id).await.unwrap()
    }
}

#[fixture]
#[once]
pub fn tracing_setup() -> () {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::DEBUG)
        .init();
    info!("tracing initialized");
}

#[fixture]
pub async fn repos(_tracing_setup: &()) -> (Arc<dyn TransactionRepo>, Arc<dyn UserRepo>) {
    let config = fs::read_to_string("config_test.toml").unwrap();
    let config: TestConfig = toml::from_str(config.as_str()).unwrap();

    create_repos(config.database_url, 1).await
}
