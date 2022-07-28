use actix_web::body::BoxBody;
use actix_web::{HttpResponse, ResponseError};
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
