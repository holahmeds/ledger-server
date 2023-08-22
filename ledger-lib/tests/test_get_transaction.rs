extern crate futures_util;
extern crate serde_json;

use std::collections::HashSet;
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
use ledger_repo::transaction_repo::{NewTransaction, Transaction, TransactionRepo};
use ledger_repo::user_repo::UserRepo;
use utils::repos;
use utils::tracing_setup;
use utils::TestUser;

#[macro_use]
mod utils;

#[instrument(skip(repos))]
#[rstest]
#[actix_rt::test]
async fn test_get_transaction(
    _tracing_setup: &(),
    repos: (Arc<dyn TransactionRepo>, Arc<dyn UserRepo>),
) {
    let (transaction_repo, user_repo) = repos;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transaction = NewTransaction::new(
        "Misc".to_string(),
        Some("Bob".to_string()),
        None,
        NaiveDate::from_str("2021-06-09").unwrap(),
        Decimal::from(100),
        HashSet::new(),
    );
    let transaction: Transaction = create_transaction!(&service, new_transaction);

    let request = TestRequest::get()
        .uri(format!("/transactions/{}", transaction.id).as_str())
        .to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let returned_transaction = test::read_body_json(response).await;
    assert_eq!(transaction, returned_transaction);

    test_user.delete().await
}

#[instrument(skip(repos))]
#[rstest]
#[actix_rt::test]
async fn test_get_invalid_transaction(
    _tracing_setup: &(),
    repos: (Arc<dyn TransactionRepo>, Arc<dyn UserRepo>),
) {
    let (transaction_repo, user_repo) = repos;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let request = TestRequest::get()
        .uri(format!("/transactions/{}", 0).as_str()) // non-existent transaction ID
        .to_request();
    let response = test::call_service(&service, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    test_user.delete().await
}
