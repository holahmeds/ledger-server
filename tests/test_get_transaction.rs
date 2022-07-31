extern crate futures_util;
extern crate serde_json;

use std::str::FromStr;

use actix_web::http::StatusCode;
use actix_web::test;
use actix_web::test::TestRequest;
use actix_web::web;
use actix_web::web::Data;
use actix_web::App;
use chrono::NaiveDate;
use rstest::rstest;
use rust_decimal::Decimal;
use tracing::instrument;

use crate::utils::mock::MockAuthentication;
use ledger::transaction::{handlers, NewTransaction, Transaction};
use ledger::user::UserId;
use ledger::DbPool;
use utils::database_pool;

mod utils;

#[instrument(skip(database_pool))]
#[rstest]
#[actix_rt::test]
async fn test_get_transaction(database_pool: &DbPool) {
    let user_id: UserId = "test-user".into();
    utils::create_user(database_pool, &user_id);

    let state = Data::new(database_pool.clone());
    let app = App::new().app_data(state).service(
        web::scope("/transactions")
            .service(handlers::get_transaction)
            .service(handlers::create_new_transaction)
            .service(handlers::delete_transaction)
            .wrap(MockAuthentication {
                user_id: user_id.clone(),
            }),
    );
    let service = test::init_service(app).await;

    let transaction: Transaction = {
        let new_transaction = NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("2021-06-09").unwrap(),
            Decimal::from_str("100").unwrap(),
            vec![],
        );
        let request = TestRequest::post()
            .uri("/transactions")
            .set_json(&new_transaction)
            .to_request();
        let response = test::call_service(&service, request).await;
        test::read_body_json(response).await
    };

    let request = TestRequest::get()
        .uri(format!("/transactions/{}", transaction.id).as_str())
        .to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let returned_transaction = test::read_body_json(response).await;
    assert_eq!(transaction, returned_transaction);

    let delete_request = TestRequest::delete()
        .uri(format!("/transactions/{}", transaction.id).as_str())
        .to_request();
    let delete_response = test::call_service(&service, delete_request).await;
    assert!(delete_response.status().is_success());

    utils::delete_user(database_pool, &user_id);
}

#[instrument(skip(database_pool))]
#[rstest]
#[actix_rt::test]
async fn test_get_invalid_transaction(database_pool: &DbPool) {
    let user_id: UserId = "test-user2".into();
    utils::create_user(database_pool, &user_id);

    let state = Data::new(database_pool.clone());
    let app = App::new()
        .app_data(state)
        .service(web::scope("/").service(handlers::get_transaction))
        .wrap(MockAuthentication {
            user_id: user_id.clone(),
        });
    let service = test::init_service(app).await;

    let request = TestRequest::get()
        .uri(format!("/{}", 0).as_str()) // non-existent transaction ID
        .to_request();
    let response = test::call_service(&service, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    utils::delete_user(database_pool, &user_id);
}
