use crate::auth::jwt::JWTAuth;
use crate::auth::password;
use crate::error::HandlerError;
use crate::user::models::User;
use crate::user::UserId;
use crate::{user, DbPool};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct UserCredentials {
    pub id: UserId,
    pub password: String,
}

#[post("/signup")]
pub async fn signup(
    pool: web::Data<DbPool>,
    credentials: web::Json<UserCredentials>,
) -> Result<impl Responder, HandlerError> {
    let credentials = credentials.into_inner();
    let password_hash = password::encode_password(credentials.password)?;

    web::block(move || {
        user::models::create_user(
            &pool,
            User {
                id: credentials.id,
                password_hash,
            },
        )
    })
    .await
    .context("Blocking error")??;

    Ok(HttpResponse::Ok())
}

#[post("/get_token")]
pub async fn get_token(
    pool: web::Data<DbPool>,
    credentials: web::Json<UserCredentials>,
    req: HttpRequest,
) -> Result<impl Responder, HandlerError> {
    let credentials = credentials.into_inner();

    let user = web::block(move || user::models::get_user(&pool, &credentials.id))
        .await
        .context("Blocking error")??;

    let matched = password::verify_password(credentials.password, user.password_hash)?;
    if matched {
        let jwt_auth = req.app_data::<JWTAuth>().unwrap();
        Ok(HttpResponse::Ok().body(jwt_auth.create_token(user.id)))
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}
