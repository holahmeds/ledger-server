mod transaction_repo;
mod user_repo;

use crate::transaction_repo::TransactionRepo;
use crate::user_repo::UserRepo;
use crate::HealthCheck;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::{query, Pool, Postgres};
use std::sync::Arc;

#[derive(Clone)]
pub struct SQLxRepo {
    pool: Pool<Postgres>,
}

impl SQLxRepo {
    pub fn from_pool(pool: Pool<Postgres>) -> SQLxRepo {
        SQLxRepo { pool }
    }

    pub async fn new(database_url: String, max_pool_size: u32) -> Result<SQLxRepo> {
        let pool = PgPoolOptions::new()
            .max_connections(max_pool_size)
            .connect(&database_url)
            .await
            .context("Unable to connect to the database")?;

        Ok(SQLxRepo { pool })
    }
}

#[async_trait]
impl HealthCheck for SQLxRepo {
    async fn check(&self) -> bool {
        query("SELECT 1").execute(&self.pool).await.is_ok()
    }
}

pub async fn create_repos(
    database_url: String,
    max_pool_size: u32,
) -> (Arc<dyn TransactionRepo>, Arc<dyn UserRepo>) {
    let repo = SQLxRepo::new(database_url, max_pool_size).await.unwrap();
    (Arc::new(repo.clone()), Arc::new(repo))
}
