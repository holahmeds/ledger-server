extern crate futures_util;
extern crate serde_json;

use std::str::FromStr;
use std::sync::Arc;

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
use ledger::user::UserRepo;
use utils::transaction_repo;
use utils::user_repo;
use utils::TestUser;

#[macro_use]
mod utils;

#[instrument(skip(transaction_repo, user_repo))]
#[rstest]
#[actix_rt::test]
async fn test_get_all_transactions(
    transaction_repo: Arc<dyn TransactionRepo>,
    user_repo: Box<dyn UserRepo>,
) {
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
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

#[instrument(skip(transaction_repo, user_repo))]
#[rstest]
#[actix_rt::test]
async fn test_transactions_sorted(
    transaction_repo: Arc<dyn TransactionRepo>,
    user_repo: Box<dyn UserRepo>,
) {
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
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

#[instrument(skip(transaction_repo, user_repo))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_category(
    transaction_repo: Arc<dyn TransactionRepo>,
    user_repo: Box<dyn UserRepo>,
) {
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
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

#[instrument(skip(transaction_repo, user_repo))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_transactee(
    transaction_repo: Arc<dyn TransactionRepo>,
    user_repo: Box<dyn UserRepo>,
) {
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
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

#[instrument(skip(transaction_repo, user_repo))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_from(
    transaction_repo: Arc<dyn TransactionRepo>,
    user_repo: Box<dyn UserRepo>,
) {
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
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

#[instrument(skip(transaction_repo, user_repo))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_until(
    transaction_repo: Arc<dyn TransactionRepo>,
    user_repo: Box<dyn UserRepo>,
) {
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
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
