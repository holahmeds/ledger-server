use crate::auth::jwt::JWTAuth;
use crate::auth::password;
use crate::error::HandlerError;
use crate::user::UserId;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use ledger_repo::user_repo::User;
use ledger_repo::user_repo::UserRepo;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct UserCredentials {
    pub id: UserId,
    pub password: String,
}

#[post("/signup")]
pub async fn signup(
    user_repo: web::Data<Arc<dyn UserRepo>>,
    credentials: web::Json<UserCredentials>,
) -> Result<impl Responder, HandlerError> {
    let credentials = credentials.into_inner();
    let password_hash = password::encode_password(credentials.password)?;

    user_repo
        .create_user(User {
            id: credentials.id,
            password_hash,
        })
        .await?;

    Ok(HttpResponse::Ok())
}

#[post("/get_token")]
pub async fn get_token(
    user_repo: web::Data<Arc<dyn UserRepo>>,
    credentials: web::Json<UserCredentials>,
    req: HttpRequest,
) -> Result<impl Responder, HandlerError> {
    let credentials = credentials.into_inner();

    let user = user_repo.get_user(&credentials.id).await?;

    let matched = password::verify_password(credentials.password, user.password_hash)?;
    if matched {
        let jwt_auth = req.app_data::<JWTAuth>().unwrap();
        Ok(HttpResponse::Ok().body(jwt_auth.create_token(user.id)))
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}
