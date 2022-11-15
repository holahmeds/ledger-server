mod transaction_repo;
mod user_repo;

use crate::repo::sqlx::transaction_repo::SQLxTransactionRepo;
use crate::repo::sqlx::user_repo::SQLxUserRepo;
use crate::repo::transaction_repo::TransactionRepo;
use crate::repo::user_repo::UserRepo;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

pub async fn create_repos(
    database_url: String,
    max_pool_size: u32,
) -> (Arc<dyn TransactionRepo>, Arc<dyn UserRepo>) {
    let pool = PgPoolOptions::new()
        .max_connections(max_pool_size)
        .connect(&database_url)
        .await
        .unwrap();

    let transaction_repo = SQLxTransactionRepo::new(pool.clone());
    let user_repo = SQLxUserRepo::new(pool);
    (Arc::new(transaction_repo), Arc::new(user_repo))
}
