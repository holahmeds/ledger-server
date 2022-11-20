use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait UserRepo: Sync + Send {
    async fn get_user(&self, user_id: &str) -> Result<User, UserRepoError>;
    async fn create_user(&self, user: User) -> Result<(), UserRepoError>;
    async fn update_password_hash(
        &self,
        user_id: &str,
        password_hash: &str,
    ) -> Result<(), UserRepoError>;
    async fn delete_user(&self, user_id: &str) -> Result<(), UserRepoError>;
}

pub struct User {
    pub id: String,
    pub password_hash: String,
}

#[derive(Error, Debug)]
pub enum UserRepoError {
    #[error("User {0} not found")]
    UserNotFound(String),
    #[error("User {0} already exists")]
    UserAlreadyExists(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
