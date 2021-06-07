#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate diesel;
extern crate serde;

use std::env;

use actix_web::{App, web};
use actix_web::HttpServer;
use diesel::PgConnection;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;

pub mod models;
pub mod schema;
pub mod transaction_handlers;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL not found in environment variables");
    let manager: ConnectionManager<diesel::PgConnection> = ConnectionManager::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Unable to build database pool");

    HttpServer::new(move || {
        App::new().data(pool.clone()).service(
            web::scope("/transactions")
                .service(transaction_handlers::get_transaction)
                .service(transaction_handlers::get_all_transactions)
                .service(transaction_handlers::create_new_transaction)
                .service(transaction_handlers::update_transaction)
                .service(transaction_handlers::delete_transaction),
        )
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
