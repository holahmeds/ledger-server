use crate::error::HandlerError;
use crate::user::{models, NewUser};
use crate::{auth, DbPool};
use actix_web::{web, HttpResponse, Responder};

#[put("")]
pub async fn update_password(
    pool: web::Data<DbPool>,
    credentials: web::Json<NewUser>,
) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    let credentials = credentials.into_inner();
    let password_hash = auth::password::encode_password(credentials.password)?;
    web::block(move || models::update_password_hash(&conn, &credentials.id, &password_hash))
        .await??;

    Ok(HttpResponse::Ok().finish())
}

#[delete("/{user_id}")]
pub async fn delete_user(
    pool: web::Data<DbPool>,
    user_id: web::Path<String>,
) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    web::block(move || models::delete_user(&conn, &user_id.into_inner())).await??;

    Ok(HttpResponse::Ok().finish())
}
