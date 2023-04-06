mod handlers;

use actix_web::{web, Scope};

pub fn transaction_service() -> Scope {
    web::scope("/transactions")
        .service(handlers::get_all_categories)
        .service(handlers::get_all_tags)
        .service(handlers::get_all_transactees)
        .service(handlers::get_balance)
        .service(handlers::get_monthly_totals)
        .service(handlers::get_transaction)
        .service(handlers::get_transactions)
        .service(handlers::create_new_transaction)
        .service(handlers::update_transaction)
        .service(handlers::delete_transaction)
}
