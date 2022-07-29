use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use crate::error::HandlerError;
use crate::user::{models, UserId};
use crate::{auth, DbPool};

#[derive(Deserialize)]
pub struct NewPassword {
    new_password: String,
}

#[put("/password")]
pub async fn update_password(
    pool: web::Data<DbPool>,
    user_id: Option<web::ReqData<UserId>>,
    credentials: web::Json<NewPassword>,
) -> Result<impl Responder, HandlerError> {
    if let None = user_id {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let conn = pool.get()?;

    let password_hash = auth::password::encode_password(credentials.into_inner().new_password)?;
    web::block(move || {
        models::update_password_hash(&conn, &user_id.unwrap().into_inner(), &password_hash)
    })
    .await??;

    Ok(HttpResponse::Ok().finish())
}

#[delete("")]
pub async fn delete_user(
    pool: web::Data<DbPool>,
    user_id: Option<web::ReqData<UserId>>,
) -> Result<impl Responder, HandlerError> {
    if let None = user_id {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let conn = pool.get()?;

    web::block(move || models::delete_user(&conn, &user_id.unwrap().into_inner())).await??;

    Ok(HttpResponse::Ok().finish())
}
