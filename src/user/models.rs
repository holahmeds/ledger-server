use anyhow::Context;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use thiserror::Error;

use crate::schema::users;
use crate::user::UserId;
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

pub fn get_user(pool: &DbPool, user_id: &str) -> Result<User, UserRepoError> {
    use crate::schema::users::dsl::users;

    let db_conn = pool.get().context("Unable to get connection from pool")?;
    let user = users.find(user_id).first(&db_conn).map_err(|e| match e {
        diesel::result::Error::NotFound => UserRepoError::UserNotFound(user_id.to_owned()),
        _ => UserRepoError::Other(e.into()),
    })?;
    Ok(user)
}

pub fn create_user(pool: &DbPool, user: User) -> Result<(), UserRepoError> {
    let db_conn = pool.get().context("Unable to get connection from pool")?;
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
}

pub fn update_password_hash(
    pool: &DbPool,
    user_id: &str,
    password_hash: &str,
) -> Result<(), UserRepoError> {
    let db_conn = pool.get().context("Unable to get connection from pool")?;
    // using get_result() so that we get NotFound if the user doesn't exist
    let _: User = diesel::update(users::table.find(user_id))
        .set(users::password_hash.eq(password_hash))
        .get_result(&db_conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => UserRepoError::UserNotFound(user_id.to_owned()),
            _ => UserRepoError::Other(e.into()),
        })?;
    Ok(())
}

pub fn delete_user(pool: &DbPool, user_id: &str) -> Result<(), UserRepoError> {
    let db_conn = pool.get().context("Unable to get connection from pool")?;
    // using get_result() so that we get NotFound if the user doesn't exist
    let _: User = diesel::delete(users::table.find(user_id))
        .get_result(&db_conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => UserRepoError::UserNotFound(user_id.to_owned()),
            _ => UserRepoError::Other(e.into()),
        })?;
    Ok(())
}
