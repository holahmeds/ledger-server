use actix_web::{web, Scope};

mod handlers;

pub fn transaction_template_service() -> Scope {
    web::scope("/templates")
        .service(handlers::create_template)
        .service(handlers::get_all_templates)
        .service(handlers::update_template)
        .service(handlers::delete_template)
}
