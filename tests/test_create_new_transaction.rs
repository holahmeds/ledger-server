extern crate futures_util;
extern crate rstest;
extern crate serde_json;

use std::str::FromStr;

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
async fn test_create_api_response(database_pool: &DbPool) {
    let user_id: UserId = "test-user".into();
    utils::create_user(database_pool, &user_id);

    let state = Data::new(database_pool.clone());
    let app = App::new().app_data(state).service(
        web::scope("/transactions")
            .service(handlers::create_new_transaction)
            .service(handlers::delete_transaction)
            .wrap(MockAuthentication {
                user_id: user_id.clone(),
            }),
    );
    let service = test::init_service(app).await;

    let new_transaction = NewTransaction::new(
        "Misc".to_string(),
        Some("Alice".to_string()),
        None,
        NaiveDate::from_str("2021-07-01").unwrap(),
        Decimal::from_str("20").unwrap(),
        vec![],
    );
    let request = TestRequest::post()
        .uri("/transactions")
        .set_json(&new_transaction)
        .to_request();
    let response = test::call_service(&service, request).await;

    assert!(
        response.status().is_success(),
        "response status was {}",
        response.status()
    );

    let response_transaction: Transaction = test::read_body_json(response).await;
    assert_eq!(new_transaction.category, response_transaction.category);
    assert_eq!(new_transaction.transactee, response_transaction.transactee);
    assert_eq!(new_transaction.category, response_transaction.category);
    assert_eq!(new_transaction.category, response_transaction.category);

    let delete_request = TestRequest::delete()
        .uri(format!("/transactions/{}", response_transaction.id).as_str())
        .to_request();
    test::call_service(&service, delete_request).await;

    utils::delete_user(database_pool, &user_id);
}
