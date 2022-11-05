use actix_web::web;
use anyhow::Context;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::result::DatabaseErrorKind;
use r2d2::PooledConnection;
use thiserror::Error;

use super::{UserId, UserRepo};
use crate::schema::users;
use crate::DbPool;

#[derive(Error, Debug)]
pub enum UserRepoError {
    #[error("User {0} not found")]
    UserNotFound(UserId),
    #[error("User {0} already exists")]
    UserAlreadyExists(UserId),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Insertable, Queryable, Identifiable, Clone)]
#[table_name = "users"]
pub struct User {
    pub id: UserId,
    pub password_hash: String,
}

pub struct DieselUserRepo {
    db_pool: DbPool,
}

impl DieselUserRepo {
    pub fn new(db_pool: DbPool) -> DieselUserRepo {
        DieselUserRepo { db_pool }
    }

    async fn block<F, R>(&self, f: F) -> Result<R, UserRepoError>
    where
        F: FnOnce(PooledConnection<ConnectionManager<PgConnection>>) -> Result<R, UserRepoError>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let pool = self.db_pool.clone();
        web::block(move || {
            let db_conn = pool.get().context("Unable to get connection from pool")?;
            f(db_conn)
        })
        .await
        .context("Blocking error")?
    }
}

#[async_trait]
impl UserRepo for DieselUserRepo {
    async fn get_user(&self, user_id: &str) -> Result<User, UserRepoError> {
        let user_id = user_id.to_owned();
        self.block(move |db_conn| {
            use crate::schema::users::dsl::users;

            let user = users.find(&user_id).first(&db_conn).map_err(|e| match e {
                diesel::result::Error::NotFound => UserRepoError::UserNotFound(user_id),
                _ => UserRepoError::Other(e.into()),
            })?;
            Ok(user)
        })
        .await
    }

    async fn create_user(&self, user: User) -> Result<(), UserRepoError> {
        self.block(move |db_conn| {
            diesel::insert_into(users::table)
                .values(&user)
                .execute(&db_conn)
                .map_err(|e| match e {
                    diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                        UserRepoError::UserAlreadyExists(user.id.to_owned())
                    }
                    _ => UserRepoError::Other(e.into()),
                })?;
            Ok(())
        })
        .await
    }

    async fn update_password_hash(
        &self,
        user_id: &str,
        password_hash: &str,
    ) -> Result<(), UserRepoError> {
        let user_id = user_id.to_owned();
        let password_hash = password_hash.to_owned();
        self.block(move |db_conn| {
            // using get_result() so that we get NotFound if the user doesn't exist
            let _: User = diesel::update(users::table.find(&user_id))
                .set(users::password_hash.eq(password_hash))
                .get_result(&db_conn)
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => UserRepoError::UserNotFound(user_id),
                    _ => UserRepoError::Other(e.into()),
                })?;
            Ok(())
        })
        .await
    }

    async fn delete_user(&self, user_id: &str) -> Result<(), UserRepoError> {
        let user_id = user_id.to_owned();
        self.block(move |db_conn| {
            // using get_result() so that we get NotFound if the user doesn't exist
            let _: User = diesel::delete(users::table.find(&user_id))
                .get_result(&db_conn)
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => UserRepoError::UserNotFound(user_id),
                    _ => UserRepoError::Other(e.into()),
                })?;
            Ok(())
        })
        .await
    }
}
