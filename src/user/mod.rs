mod handlers;

use actix_web::{web, Scope};

pub type UserId = String;

pub fn user_service() -> Scope {
    web::scope("/user")
        .service(handlers::update_password)
        .service(handlers::delete_user)
}
