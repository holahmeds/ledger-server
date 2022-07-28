use crate::auth::{password, JWTAuth};
use crate::error::HandlerError;
use crate::user::models::User;
use crate::user::NewUser;
use crate::{user, DbPool};
use actix_web::{web, HttpRequest, HttpResponse, Responder};

#[post("/signup")]
pub async fn signup(
    pool: web::Data<DbPool>,
    new_user: web::Json<NewUser>,
) -> Result<impl Responder, HandlerError> {
    let new_user = new_user.into_inner();
    let conn = pool.get()?;

    let password_hash = password::encode_password(new_user.password)?;

    web::block(move || {
        user::models::create_user(
            &conn,
            User {
                id: new_user.id,
                password_hash,
            },
        )
    })
    .await??;

    Ok(HttpResponse::Ok())
}

#[post("/get_token")]
pub async fn get_token(
    pool: web::Data<DbPool>,
    credentials: web::Json<NewUser>,
    req: HttpRequest,
) -> Result<impl Responder, HandlerError> {
    let credentials = credentials.into_inner();
    let con = pool.get()?;

    let user = web::block(move || user::models::get_user(&con, &credentials.id)).await??;

    let matched = password::verify_password(credentials.password, user.password_hash)?;
    if matched {
        let jwt_auth = req.app_data::<JWTAuth>().unwrap();
        Ok(HttpResponse::Ok().body(jwt_auth.create_token()))
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}
