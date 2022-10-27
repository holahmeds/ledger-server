use actix_web::{web, Scope};

mod handlers;
pub mod models;

pub type UserId = String;

pub fn user_service() -> Scope {
    web::scope("/user")
        .service(handlers::update_password)
        .service(handlers::delete_user)
}
