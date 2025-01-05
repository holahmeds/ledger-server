use async_trait::async_trait;

pub mod transaction_repo;
pub mod transaction_template_repo;
pub mod user_repo;

// implementation modules
pub mod mem_repo;
pub mod sqlx_repo;

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> bool;
}
