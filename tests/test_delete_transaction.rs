use std::str::FromStr;

use actix_web::App;
use actix_web::http::StatusCode;
use actix_web::test;
use actix_web::test::TestRequest;
use actix_web::web;
use chrono::NaiveDate;
use dotenv::dotenv;
use rust_decimal::Decimal;

use ledger::models::{NewTransaction, Transaction};
use ledger::transaction_handlers;
use utils::database_pool;
use utils::map_body;

mod utils;

#[actix_rt::test]
async fn test_delete_transaction() {
    dotenv().ok();

    let pool = database_pool();

    let app = App::new().data(pool.clone()).service(
        web::scope("/")
            .service(transaction_handlers::update_transaction)
            .service(transaction_handlers::create_new_transaction)
            .service(transaction_handlers::delete_transaction),
    );
    let mut service = test::init_service(app).await;

    let new_transaction = NewTransaction::new(
        "Misc".to_string(),
        Some("Bob".to_string()),
        None,
        NaiveDate::from_str("2021-06-09").unwrap(),
        Decimal::from_str("5.10").unwrap(),
        vec!["Monthly".to_string()],
    );
    let transaction = {
        let request = TestRequest::post().set_json(&new_transaction).to_request();
        let mut response = test::call_service(&mut service, request).await;
        map_body::<Transaction>(&mut response).await
    };

    let request = TestRequest::delete()
        .uri(format!("/{}", transaction.id).as_str())
        .to_request();
    let mut response = test::call_service(&mut service, request).await;
    let deleted_transaction = map_body(&mut response).await;

    assert!(response.status().is_success());
    assert_eq!(transaction, deleted_transaction);
}

#[actix_rt::test]
async fn test_delete_invalid_transaction() {
    dotenv().ok();

    let pool = database_pool();

    let app = App::new()
        .data(pool.clone())
        .service(web::scope("/").service(transaction_handlers::delete_transaction));
    let mut service = test::init_service(app).await;

    let request = TestRequest::delete()
        .uri(format!("/{}", 0).as_str()) // non-existent transaction ID
        .to_request();
    let response = test::call_service(&mut service, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND)
}
