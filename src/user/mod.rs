use actix_web::{web, Scope};
use async_trait::async_trait;
use models::{User, UserRepoError};

mod handlers;
pub mod models;

pub type UserId = String;

pub fn user_service() -> Scope {
    web::scope("/user")
        .service(handlers::update_password)
        .service(handlers::delete_user)
}

#[async_trait]
pub trait UserRepo {
    async fn get_user(&self, user_id: &str) -> Result<User, UserRepoError>;
    async fn create_user(&self, user: User) -> Result<(), UserRepoError>;
    async fn update_password_hash(
        &self,
        user_id: &str,
        password_hash: &str,
    ) -> Result<(), UserRepoError>;
    async fn delete_user(&self, user_id: &str) -> Result<(), UserRepoError>;
}
