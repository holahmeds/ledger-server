use crate::error::HandlerError;
use crate::user::models::User;
use crate::user::{models, NewUser};
use crate::DbPool;
use actix_web::{web, HttpResponse, Responder};

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

    web::block(move || models::delete_user(&conn, &user_id.into_inner())).await??;

    Ok(HttpResponse::Ok().finish())
}
