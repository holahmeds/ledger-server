use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;

use super::UserId;
use crate::auth;
use crate::error::HandlerError;
use crate::repo::user_repo::UserRepo;

#[derive(Deserialize)]
pub struct NewPassword {
    new_password: String,
}

#[put("/password")]
pub async fn update_password(
    user_repo: web::Data<Arc<dyn UserRepo>>,
    user_id: Option<web::ReqData<UserId>>,
    credentials: web::Json<NewPassword>,
) -> Result<impl Responder, HandlerError> {
    if user_id.is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let password_hash = auth::password::encode_password(credentials.into_inner().new_password)?;
    user_repo
        .update_password_hash(&user_id.unwrap().into_inner(), &password_hash)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[delete("")]
pub async fn delete_user(
    user_repo: web::Data<Arc<dyn UserRepo>>,
    user_id: Option<web::ReqData<UserId>>,
) -> Result<impl Responder, HandlerError> {
    if user_id.is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    user_repo
        .delete_user(&user_id.unwrap().into_inner())
        .await?;

    Ok(HttpResponse::Ok().finish())
}
