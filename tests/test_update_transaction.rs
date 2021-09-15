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
async fn test_update_transaction() {
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
        Decimal::from_str("11.12").unwrap(),
        vec![],
    );
    let transaction = {
        let request = TestRequest::post().set_json(&new_transaction).to_request();
        let mut response = test::call_service(&mut service, request).await;
        map_body::<Transaction>(&mut response).await
    };

    let update = NewTransaction::new(
        new_transaction.category,
        Some("Alice".to_string()),
        new_transaction.note,
        new_transaction.transaction_date,
        Decimal::from_str("105").unwrap(),
        vec![],
    );
    let request = TestRequest::put()
        .uri(format!("/{}", transaction.id).as_str())
        .set_json(&update)
        .to_request();
    let mut response = test::call_service(&mut service, request).await;
    let updated_transaction = map_body::<Transaction>(&mut response).await;

    assert_eq!(transaction.id, updated_transaction.id);
    assert_ne!(transaction, updated_transaction);
    assert_eq!(updated_transaction.transactee, update.transactee);

    let delete_request = TestRequest::delete()
        .uri(format!("/{}", transaction.id).as_str())
        .to_request();
    test::call_service(&mut service, delete_request).await;
}

#[actix_rt::test]
async fn test_update_tags() {
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
        Decimal::from_str("11.12").unwrap(),
        vec!["tag1".to_string(), "tag2".to_string()],
    );
    let transaction = {
        let request = TestRequest::post().set_json(&new_transaction).to_request();
        let mut response = test::call_service(&mut service, request).await;
        map_body::<Transaction>(&mut response).await
    };

    let update = NewTransaction::new(
        new_transaction.category,
        Some("Alice".to_string()),
        new_transaction.note,
        new_transaction.transaction_date,
        Decimal::from_str("105").unwrap(),
        vec!["tag2".to_string(), "tag3".to_string()],
    );
    let request = TestRequest::put()
        .uri(format!("/{}", transaction.id).as_str())
        .set_json(&update)
        .to_request();
    let mut response = test::call_service(&mut service, request).await;
    let updated_transaction = map_body::<Transaction>(&mut response).await;

    assert_eq!(transaction.id, updated_transaction.id);
    assert_ne!(transaction, updated_transaction);
    assert_eq!(updated_transaction.transactee, update.transactee);
    assert_eq!(updated_transaction.tags, update.tags);

    let delete_request = TestRequest::delete()
        .uri(format!("/{}", transaction.id).as_str())
        .to_request();
    test::call_service(&mut service, delete_request).await;
}

#[actix_rt::test]
async fn test_update_invalid_transaction() {
    dotenv().ok();

    let pool = database_pool();

    let app = App::new()
        .data(pool.clone())
        .service(web::scope("/").service(transaction_handlers::update_transaction));
    let mut service = test::init_service(app).await;

    let update = NewTransaction::new(
        "Misc".to_string(),
        Some("Bob".to_string()),
        None,
        NaiveDate::from_str("2021-06-09").unwrap(),
        Decimal::from_str("321").unwrap(),
        vec![],
    );
    let request = TestRequest::put()
        .uri(format!("/{}", 0).as_str()) // non-existent transaction ID
        .set_json(&update)
        .to_request();
    let response = test::call_service(&mut service, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND)
}
