use actix_web::http::StatusCode;
use actix_web::ResponseError;
use ledger_repo::transaction_repo::TransactionRepoError;
use ledger_repo::user_repo::UserRepoError;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Authentication error")]
    AuthError(#[from] argon2::Error),
    #[error("Internal Server Error")]
    OtherError(#[from] anyhow::Error),
    #[error(transparent)]
    TransactionNotFoundError(TransactionRepoError),
    #[error(transparent)]
    UserNotFoundError(UserRepoError),
    #[error(transparent)]
    UserAlreadyExists(UserRepoError),
    #[error("{0}")]
    BadRequest(String),
}

impl From<TransactionRepoError> for HandlerError {
    fn from(e: TransactionRepoError) -> Self {
        match e {
            TransactionRepoError::TransactionNotFound(_) => {
                HandlerError::TransactionNotFoundError(e)
            }
            TransactionRepoError::Other(e) => HandlerError::OtherError(e),
        }
    }
}

impl From<UserRepoError> for HandlerError {
    fn from(e: UserRepoError) -> Self {
        match e {
            UserRepoError::UserNotFound(_) => HandlerError::UserNotFoundError(e),
            UserRepoError::UserAlreadyExists(_) => HandlerError::UserAlreadyExists(e),
            UserRepoError::Other(e) => HandlerError::OtherError(e),
        }
    }
}

impl ResponseError for HandlerError {
    fn status_code(&self) -> StatusCode {
        match self {
            HandlerError::TransactionNotFoundError(_) | HandlerError::UserNotFoundError(_) => {
                StatusCode::NOT_FOUND
            }
            HandlerError::UserAlreadyExists(_) => StatusCode::CONFLICT,
            HandlerError::BadRequest(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
