use crate::user_repo::{User, UserRepo, UserRepoError};
use anyhow::Context;
use async_trait::async_trait;
use sqlx::{query, query_as, Pool, Postgres};

pub struct SQLxUserRepo {
    pool: Pool<Postgres>,
}

impl SQLxUserRepo {
    pub fn new(pool: Pool<Postgres>) -> SQLxUserRepo {
        SQLxUserRepo { pool }
    }
}

#[async_trait]
impl UserRepo for SQLxUserRepo {
    async fn get_user(&self, user_id: &str) -> Result<User, UserRepoError> {
        let user: Option<User> = query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_optional(&self.pool)
            .await
            .with_context(|| format!("Unable to get user {}", user_id))?;
        user.ok_or_else(|| UserRepoError::UserNotFound(user_id.to_owned()))
    }

    async fn create_user(&self, user: User) -> Result<(), UserRepoError> {
        let result = query!(
            "INSERT INTO users(id, password_hash) VALUES($1, $2) ON CONFLICT DO NOTHING",
            &user.id,
            user.password_hash
        )
        .execute(&self.pool)
        .await
        .with_context(|| format!("Unable to create user {}", user.id))?;
        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err(UserRepoError::UserAlreadyExists(user.id))
        }
    }

    async fn update_password_hash(
        &self,
        user_id: &str,
        password_hash: &str,
    ) -> Result<(), UserRepoError> {
        let result = query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            password_hash,
            user_id
        )
        .execute(&self.pool)
        .await
        .with_context(|| format!("Unable to update password for {}", user_id))?;
        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err(UserRepoError::UserNotFound(user_id.to_owned()))
        }
    }

    async fn delete_user(&self, user_id: &str) -> Result<(), UserRepoError> {
        let result = query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("Unable to delete user {}", user_id))?;
        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err(UserRepoError::UserNotFound(user_id.to_owned()))
        }
    }
}
