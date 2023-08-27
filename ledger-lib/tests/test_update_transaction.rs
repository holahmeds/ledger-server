use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::test;
use actix_web::test::{read_body_json, TestRequest};
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
async fn test_update_transaction(
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
        Decimal::from_str("11.12").unwrap(),
        HashSet::new(),
    );
    let transaction: Transaction = {
        let request = TestRequest::post()
            .uri("/transactions")
            .set_json(&new_transaction)
            .to_request();
        let response = test::call_service(&service, request).await;
        read_body_json(response).await
    };

    let update = NewTransaction::new(
        new_transaction.category,
        Some("Alice".to_string()),
        new_transaction.note,
        new_transaction.date,
        Decimal::from(105),
        HashSet::new(),
    );
    let request = TestRequest::put()
        .uri(format!("/transactions/{}", transaction.id).as_str())
        .set_json(&update)
        .to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let updated_transaction: Transaction = read_body_json(response).await;
    assert_eq!(transaction.id, updated_transaction.id);
    assert_ne!(transaction, updated_transaction);
    assert_eq!(updated_transaction.transactee, update.transactee);

    test_user.delete().await
}

#[instrument(skip(repos))]
#[rstest]
#[actix_rt::test]
async fn test_update_tags(
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
        Decimal::from_str("11.12").unwrap(),
        HashSet::from(["tag1".to_string(), "tag2".to_string()]),
    );
    let transaction: Transaction = create_transaction!(&service, new_transaction);

    let update = NewTransaction::new(
        new_transaction.category,
        Some("Alice".to_string()),
        new_transaction.note,
        new_transaction.date,
        Decimal::from(105),
        HashSet::from(["tag2".to_string(), "tag3".to_string()]),
    );
    let request = TestRequest::put()
        .uri(format!("/transactions/{}", transaction.id).as_str())
        .set_json(&update)
        .to_request();
    let response = test::call_service(&service, request).await;
    let updated_transaction: Transaction = read_body_json(response).await;

    assert_eq!(transaction.id, updated_transaction.id);
    assert_ne!(transaction, updated_transaction);
    assert_eq!(updated_transaction.transactee, update.transactee);
    assert_eq!(updated_transaction.tags, update.tags);

    test_user.delete().await
}

#[instrument(skip(repos))]
#[rstest]
#[actix_rt::test]
async fn test_update_invalid_transaction(
    _tracing_setup: &(),
    repos: (Arc<dyn TransactionRepo>, Arc<dyn UserRepo>),
) {
    let (transaction_repo, user_repo) = repos;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let update = NewTransaction::new(
        "Misc".to_string(),
        Some("Bob".to_string()),
        None,
        NaiveDate::from_str("2021-06-09").unwrap(),
        Decimal::from(321),
        HashSet::new(),
    );
    let request = TestRequest::put()
        .uri(format!("/transactions/{}", 0).as_str()) // non-existent transaction ID
        .set_json(&update)
        .to_request();
    let response = test::call_service(&service, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    test_user.delete().await
}
