use ledger_repo::transaction_repo::TransactionRepo;
use ledger_repo::user_repo::UserRepo;
use serde::Deserialize;
use std::fs;
use std::sync::Arc;

#[derive(Deserialize)]
struct TestConfig {
    database_url: String,
}

#[derive(Debug)]
pub enum RepoType {
    Diesel,
    SQLx,
    Mem,
}

pub async fn build_repos(repo_type: RepoType) -> (Arc<dyn TransactionRepo>, Arc<dyn UserRepo>) {
    let config = fs::read_to_string("config_test.toml").unwrap();
    let config: TestConfig = toml::from_str(config.as_str()).unwrap();

    match repo_type {
        RepoType::Diesel => ledger_repo::diesel_repo::create_repos(config.database_url, 1, false),
        RepoType::SQLx => ledger_repo::sqlx_repo::create_repos(config.database_url, 1).await,
        RepoType::Mem => ledger_repo::mem_repo::create_repos(),
    }
}
