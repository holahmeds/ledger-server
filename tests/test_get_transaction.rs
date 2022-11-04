extern crate futures_util;
extern crate serde_json;

use std::str::FromStr;
use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::test;
use actix_web::test::TestRequest;
use actix_web::web::Data;
use actix_web::App;
use chrono::NaiveDate;
use rstest::rstest;
use rust_decimal::Decimal;
use tracing::instrument;

use crate::utils::mock::MockAuthentication;
use ledger::transaction::{NewTransaction, Transaction, TransactionRepo};
use ledger::DbPool;
use utils::database_pool;
use utils::test_user;
use utils::transaction_repo;
use utils::TestUser;

#[macro_use]
mod utils;

#[instrument(skip(database_pool, transaction_repo, test_user))]
#[rstest]
#[actix_rt::test]
async fn test_get_transaction(
    database_pool: &DbPool,
    transaction_repo: Arc<dyn TransactionRepo>,
    test_user: TestUser,
) {
    let app = build_app!(database_pool, transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transaction = NewTransaction::new(
        "Misc".to_string(),
        Some("Bob".to_string()),
        None,
        NaiveDate::from_str("2021-06-09").unwrap(),
        Decimal::from_str("100").unwrap(),
        vec![],
    );
    let transaction: Transaction = create_transaction!(&service, new_transaction);

    let request = TestRequest::get()
        .uri(format!("/transactions/{}", transaction.id).as_str())
        .to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let returned_transaction = test::read_body_json(response).await;
    assert_eq!(transaction, returned_transaction);
}

#[instrument(skip(database_pool, transaction_repo, test_user))]
#[rstest]
#[actix_rt::test]
async fn test_get_invalid_transaction(
    database_pool: &DbPool,
    transaction_repo: Arc<dyn TransactionRepo>,
    test_user: TestUser,
) {
    let app = build_app!(database_pool, transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let request = TestRequest::get()
        .uri(format!("/transactions/{}", 0).as_str()) // non-existent transaction ID
        .to_request();
    let response = test::call_service(&service, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
