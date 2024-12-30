use ledger_repo::transaction_repo::TransactionRepo;
use ledger_repo::transaction_template_repo::TransactionTemplateRepo;
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
    SQLx,
    Mem,
}

pub async fn build_repos(
    repo_type: RepoType,
) -> (
    Arc<dyn UserRepo>,
    Arc<dyn TransactionRepo>,
    Arc<dyn TransactionTemplateRepo>,
) {
    let config = fs::read_to_string("config_test.toml").unwrap();
    let config: TestConfig = toml::from_str(config.as_str()).unwrap();

    match repo_type {
        RepoType::SQLx => ledger_repo::sqlx_repo::create_repos(config.database_url, 1).await,
        RepoType::Mem => ledger_repo::mem_repo::create_repos(),
    }
}
