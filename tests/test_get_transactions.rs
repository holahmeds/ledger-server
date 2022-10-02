extern crate futures_util;
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
use ledger::DbPool;
use utils::database_pool;
use utils::test_user;
use utils::TestUser;

#[macro_use]
mod utils;

#[instrument(skip(database_pool, test_user))]
#[rstest]
#[actix_rt::test]
async fn test_get_all_transactions(database_pool: &DbPool, test_user: TestUser) {
    let app = build_app!(database_pool, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Misc".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from_str("10").unwrap(),
            vec![],
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from_str("15").unwrap(),
            vec!["loan".to_string()],
        ),
    ];

    let mut inserted_transactions: Vec<Transaction> = vec![];
    for t in new_transactions {
        let transaction = create_transaction!(&service, t);
        inserted_transactions.push(transaction);
    }

    let request = TestRequest::get().uri("/transactions").to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let transactions: Vec<Transaction> = test::read_body_json(response).await;
    assert_eq!(inserted_transactions, transactions);
}

#[instrument(skip(database_pool, test_user))]
#[rstest]
#[actix_rt::test]
async fn test_transactions_sorted(database_pool: &DbPool, test_user: TestUser) {
    let app = build_app!(database_pool, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Misc".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from_str("10").unwrap(),
            vec![],
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from_str("15").unwrap(),
            vec!["loan".to_string()],
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("2022-08-02").unwrap(),
            Decimal::from_str("20").unwrap(),
            vec![],
        ),
    ];

    for t in new_transactions {
        let _transaction: Transaction = create_transaction!(&service, t);
    }

    let request = TestRequest::get().uri("/transactions").to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let transactions: Vec<Transaction> = test::read_body_json(response).await;
    assert!(
        transactions.windows(2).all(|w| w[0] >= w[1]),
        "transactions not sorted"
    );
}

#[instrument(skip(database_pool, test_user))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_category(database_pool: &DbPool, test_user: TestUser) {
    let app = build_app!(database_pool, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Loan".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from_str("10").unwrap(),
            vec![],
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from_str("15").unwrap(),
            vec!["loan".to_string()],
        ),
    ];

    let mut inserted_transactions: Vec<Transaction> = vec![];
    for t in new_transactions {
        let transaction = create_transaction!(&service, t);
        inserted_transactions.push(transaction);
    }

    let request = TestRequest::get()
        .uri("/transactions?category=Loan")
        .to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let transactions: Vec<Transaction> = test::read_body_json(response).await;
    assert!(transactions.iter().all(|t| t.category == "Loan"));
}

#[instrument(skip(database_pool, test_user))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_transactee(database_pool: &DbPool, test_user: TestUser) {
    let app = build_app!(database_pool, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Loan".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from_str("10").unwrap(),
            vec![],
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from_str("15").unwrap(),
            vec!["loan".to_string()],
        ),
    ];

    let mut inserted_transactions: Vec<Transaction> = vec![];
    for t in new_transactions {
        let transaction = create_transaction!(&service, t);
        inserted_transactions.push(transaction);
    }

    let request = TestRequest::get()
        .uri("/transactions?transactee=Alice")
        .to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let transactions: Vec<Transaction> = test::read_body_json(response).await;
    assert!(transactions
        .iter()
        .all(|t| t.transactee == Some("Alice".to_owned())));
}

#[instrument(skip(database_pool, test_user))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_from(database_pool: &DbPool, test_user: TestUser) {
    let app = build_app!(database_pool, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Loan".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from_str("10").unwrap(),
            vec![],
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from_str("15").unwrap(),
            vec!["loan".to_string()],
        ),
    ];

    let mut inserted_transactions: Vec<Transaction> = vec![];
    for t in new_transactions {
        let transaction = create_transaction!(&service, t);
        inserted_transactions.push(transaction);
    }

    let request = TestRequest::get()
        .uri("/transactions?from=2021-01-01")
        .to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let transactions: Vec<Transaction> = test::read_body_json(response).await;
    assert!(transactions
        .iter()
        .all(|t| t.date > NaiveDate::from_str("2021-01-01").unwrap()));
}

#[instrument(skip(database_pool, test_user))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_until(database_pool: &DbPool, test_user: TestUser) {
    let app = build_app!(database_pool, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Loan".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from_str("10").unwrap(),
            vec![],
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from_str("15").unwrap(),
            vec!["loan".to_string()],
        ),
    ];

    let mut inserted_transactions: Vec<Transaction> = vec![];
    for t in new_transactions {
        let transaction = create_transaction!(&service, t);
        inserted_transactions.push(transaction);
    }

    let request = TestRequest::get()
        .uri("/transactions?until=2021-01-01")
        .to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let transactions: Vec<Transaction> = test::read_body_json(response).await;
    assert!(transactions
        .iter()
        .all(|t| t.date < NaiveDate::from_str("2021-01-01").unwrap()));
}