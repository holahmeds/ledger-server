use crate::user::models::User;
use crate::user::{models, NewUser};
use crate::DbPool;
use actix_web::body::BoxBody;
use actix_web::{web, HttpResponse, Responder, ResponseError};
use diesel::result::DatabaseErrorKind;
use std::fmt::{Debug, Display, Formatter};

pub enum HandlerError {
    DbError(diesel::result::Error),
    PoolError(r2d2::Error),
    BlockingError(actix_web::error::BlockingError),
}

impl From<diesel::result::Error> for HandlerError {
    fn from(e: diesel::result::Error) -> Self {
        HandlerError::DbError(e)
    }
}

impl From<r2d2::Error> for HandlerError {
    fn from(e: r2d2::Error) -> Self {
        HandlerError::PoolError(e)
    }
}

impl From<actix_web::error::BlockingError> for HandlerError {
    fn from(e: actix_web::error::BlockingError) -> Self {
        HandlerError::BlockingError(e)
    }
}

impl Debug for HandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HandlerError::DbError(e) => f.write_fmt(format_args!("DbError({})", e)),
            HandlerError::PoolError(e) => f.write_fmt(format_args!("PoolError({})", e)),
            HandlerError::BlockingError(_) => f.write_str("BlockingError"),
        }
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HandlerError::DbError(e) => f.write_fmt(format_args!("DbError({})", e)),
            HandlerError::PoolError(e) => f.write_fmt(format_args!("PoolError({})", e)),
            HandlerError::BlockingError(_) => f.write_str("BlockingError"),
        }
    }
}

impl ResponseError for HandlerError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            HandlerError::DbError(diesel::result::Error::NotFound) => {
                HttpResponse::NotFound().finish()
            }
            HandlerError::DbError(diesel::result::Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _,
            )) => HttpResponse::Conflict().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

#[post("")]
pub async fn create_user(
    pool: web::Data<DbPool>,
    new_user: web::Json<NewUser>,
) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    let new_user = new_user.into_inner();
    let user = User {
        id: new_user.id,
        password_hash: new_user.password,
    };

    web::block(move || models::create_user(&conn, user)).await??;

    Ok(HttpResponse::Ok().finish())
}

#[put("")]
pub async fn update_password(
    pool: web::Data<DbPool>,
    credentials: web::Json<NewUser>,
) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    let credentials = credentials.into_inner();
    web::block(move || models::update_password_hash(&conn, &credentials.id, &credentials.password))
        .await??;

    Ok(HttpResponse::Ok().finish())
}

#[delete("/{user_id}")]
pub async fn delete_user(
    pool: web::Data<DbPool>,
    user_id: web::Path<String>,
) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    web::block(move || models::delete_user(&conn, &*user_id.into_inner())).await??;

    Ok(HttpResponse::Ok().finish())
}
