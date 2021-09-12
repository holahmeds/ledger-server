#[macro_use]
extern crate tracing;

use std::env;

use actix_web::{App, web};
use actix_web::HttpServer;
use actix_web::middleware::Logger;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use tracing::Level;

use ledger::transaction_handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    info!("tracing initialized");

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL not found in environment variables");
    let manager: ConnectionManager<diesel::PgConnection> = ConnectionManager::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Unable to build database pool");

    HttpServer::new(move || {
        App::new().wrap(Logger::default()).data(pool.clone()).service(
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
