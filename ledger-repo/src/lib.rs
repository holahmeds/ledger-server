#[cfg(feature = "diesel-repo")]
#[macro_use]
extern crate diesel_migrations;

use async_trait::async_trait;

pub mod transaction_repo;
pub mod user_repo;

// implementation modules
#[cfg(feature = "diesel-repo")]
pub mod diesel_repo;
pub mod mem_repo;
pub mod sqlx_repo;

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> bool;
}
