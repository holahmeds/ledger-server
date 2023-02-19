use super::schema::users;
use super::DbPool;
use crate::user_repo::{User, UserRepo, UserRepoError};
use actix_web::web;
use anyhow::Context;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::result::DatabaseErrorKind;
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use r2d2::PooledConnection;
use tracing::instrument;

#[derive(Insertable, Queryable, Identifiable, Clone)]
#[diesel(table_name = users)]
pub struct UserEntry {
    pub id: String,
    pub password_hash: String,
}

impl From<UserEntry> for User {
    fn from(u: UserEntry) -> Self {
        User::new(u.id, u.password_hash)
    }
}

impl From<User> for UserEntry {
    fn from(u: User) -> Self {
        UserEntry {
            id: u.id,
            password_hash: u.password_hash,
        }
    }
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
        F: FnOnce(
                &mut PooledConnection<ConnectionManager<PgConnection>>,
            ) -> Result<R, UserRepoError>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let pool = self.db_pool.clone();
        web::block(move || {
            let mut db_conn = pool.get().context("Unable to get connection from pool")?;
            f(&mut db_conn)
        })
        .await
        .context("Blocking error")?
    }
}

#[async_trait]
impl UserRepo for DieselUserRepo {
    #[instrument(skip(self))]
    async fn get_user(&self, user_id: &str) -> Result<User, UserRepoError> {
        let user_id = user_id.to_owned();
        self.block(move |db_conn| {
            use crate::diesel_repo::schema::users::dsl::users;
            use diesel::{QueryDsl, RunQueryDsl};

            let user: UserEntry = users.find(&user_id).first(db_conn).map_err(|e| match e {
                diesel::result::Error::NotFound => UserRepoError::UserNotFound(user_id),
                _ => UserRepoError::Other(e.into()),
            })?;
            Ok(user.into())
        })
        .await
    }

    #[instrument(skip(self))]
    async fn create_user(&self, user: User) -> Result<(), UserRepoError> {
        let user_entry: UserEntry = user.into();
        self.block(move |db_conn| {
            diesel::insert_into(users::table)
                .values(&user_entry)
                .execute(db_conn)
                .map_err(|e| match e {
                    diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                        UserRepoError::UserAlreadyExists(user_entry.id.to_owned())
                    }
                    _ => UserRepoError::Other(e.into()),
                })?;

            Ok(())
        })
        .await
    }

    #[instrument(skip(self))]
    async fn update_password_hash(
        &self,
        user_id: &str,
        password_hash: &str,
    ) -> Result<(), UserRepoError> {
        let user_id = user_id.to_owned();
        let password_hash = password_hash.to_owned();
        self.block(move |db_conn| {
            // using get_result() so that we get NotFound if the user doesn't exist
            let _: UserEntry = diesel::update(users::table.find(&user_id))
                .set(users::password_hash.eq(password_hash))
                .get_result(db_conn)
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => UserRepoError::UserNotFound(user_id),
                    _ => UserRepoError::Other(e.into()),
                })?;
            Ok(())
        })
        .await
    }

    #[instrument(skip(self))]
    async fn delete_user(&self, user_id: &str) -> Result<(), UserRepoError> {
        let user_id = user_id.to_owned();
        self.block(move |db_conn| {
            // using get_result() so that we get NotFound if the user doesn't exist
            let _: UserEntry = diesel::delete(users::table.find(&user_id))
                .get_result(db_conn)
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => UserRepoError::UserNotFound(user_id),
                    _ => UserRepoError::Other(e.into()),
                })?;
            Ok(())
        })
        .await
    }
}
