extern crate futures_util;
extern crate serde_json;

use std::str::FromStr;

use actix_web::App;
use actix_web::http::StatusCode;
use actix_web::test;
use actix_web::test::TestRequest;
use actix_web::web;
use chrono::NaiveDate;
use rstest::rstest;
use rust_decimal::Decimal;
use tracing::instrument;

use ledger::{DbPool, transaction_handlers};
use ledger::models::{NewTransaction, Transaction};
use utils::database_pool;
use utils::map_body;

mod utils;

#[instrument(skip(database_pool))]
#[rstest]
#[actix_rt::test]
async fn test_get_transaction(database_pool: DbPool) {
    let app = App::new().data(database_pool.clone()).service(
        web::scope("/")
            .service(transaction_handlers::get_transaction)
            .service(transaction_handlers::create_new_transaction)
            .service(transaction_handlers::delete_transaction),
    );
    let mut service = test::init_service(app).await;

    let transaction = {
        let new_transaction = NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("2021-06-09").unwrap(),
            Decimal::from_str("100").unwrap(),
            vec![],
        );
        let request = TestRequest::post().set_json(&new_transaction).to_request();
        let mut response = test::call_service(&mut service, request).await;
        map_body::<Transaction>(&mut response).await
    };

    let request = TestRequest::get()
        .uri(format!("/{}", transaction.id).as_str())
        .to_request();
    let mut response = test::call_service(&mut service, request).await;
    let returned_transaction = map_body(&mut response).await;

    assert!(response.status().is_success());
    assert_eq!(transaction, returned_transaction);

    let delete_request = TestRequest::delete()
        .uri(format!("/{}", transaction.id).as_str())
        .to_request();
    test::call_service(&mut service, delete_request).await;
}

#[instrument(skip(database_pool))]
#[rstest]
#[actix_rt::test]
async fn test_get_invalid_transaction(database_pool: DbPool) {
    let app = App::new()
        .data(database_pool.clone())
        .service(web::scope("/").service(transaction_handlers::get_transaction));
    let mut service = test::init_service(app).await;

    let request = TestRequest::get()
        .uri(format!("/{}", 0).as_str()) // non-existent transaction ID
        .to_request();
    let response = test::call_service(&mut service, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
