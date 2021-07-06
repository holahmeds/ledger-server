extern crate futures_util;
extern crate serde_json;

use std::str::FromStr;

use actix_web::App;
use actix_web::test;
use actix_web::test::TestRequest;
use actix_web::web;
use chrono::NaiveDate;
use dotenv::dotenv;

use ledger::models::{NewTransaction, Transaction};
use ledger::transaction_handlers;
use utils::database_pool;
use utils::map_body;

mod utils;

#[actix_rt::test]
async fn test_create_api_response() {
    dotenv().ok();

    let pool = database_pool();

    let app = App::new().data(pool.clone()).service(
        web::scope("/")
            .service(transaction_handlers::create_new_transaction)
            .service(transaction_handlers::delete_transaction),
    );
    let mut service = test::init_service(app).await;

    let new_transaction = NewTransaction::new(
        "Misc".to_string(),
        Some("Alice".to_string()),
        None,
        NaiveDate::from_str("2021-07-01").unwrap(),
    );
    let request = TestRequest::post().set_json(&new_transaction).to_request();
    let mut response = test::call_service(&mut service, request).await;

    assert!(response.status().is_success());

    let response_transaction = map_body::<Transaction>(&mut response).await;
    assert_eq!(new_transaction.category, response_transaction.category);
    assert_eq!(new_transaction.transactee, response_transaction.transactee);
    assert_eq!(new_transaction.category, response_transaction.category);
    assert_eq!(new_transaction.category, response_transaction.category);

    let delete_request = TestRequest::delete()
        .uri(format!("/{}", response_transaction.id).as_str())
        .to_request();
    test::call_service(&mut service, delete_request).await;
}
