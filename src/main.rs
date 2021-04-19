#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;

pub mod models;
pub mod schema;
pub mod transaction_handlers;

#[database("postgres")]
pub struct DBConnection(diesel::PgConnection);

fn main() {
    rocket::ignite()
        .attach(DBConnection::fairing())
        .mount(
            "/transactions",
            routes![
                transaction_handlers::get_transaction,
                transaction_handlers::get_all_transactions,
                transaction_handlers::create_new_transaction,
                transaction_handlers::update_transaction,
                transaction_handlers::delete_transaction
            ],
        )
        .launch();
}
