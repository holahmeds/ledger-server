extern crate futures_util;
extern crate serde_json;

use std::collections::HashSet;
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
use ledger_repo::transaction_repo::{NewTransaction, Transaction, TransactionRepo};
use ledger_repo::transaction_template_repo::TransactionTemplateRepo;
use ledger_repo::user_repo::UserRepo;
use utils::repos;
use utils::tracing_setup;
use utils::TestUser;

#[macro_use]
mod utils;

#[instrument(skip(repos))]
#[rstest]
#[actix_rt::test]
async fn test_get_all_transactions(
    _tracing_setup: &(),
    repos: (
        Arc<dyn UserRepo>,
        Arc<dyn TransactionRepo>,
        Arc<dyn TransactionTemplateRepo>,
    ),
) {
    let (user_repo, transaction_repo, _template_repo) = repos;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Misc".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from(10),
            HashSet::new(),
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from(15),
            HashSet::new(),
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

    test_user.delete().await
}

#[instrument(skip(repos))]
#[rstest]
#[actix_rt::test]
async fn test_transactions_sorted(
    _tracing_setup: &(),
    repos: (
        Arc<dyn UserRepo>,
        Arc<dyn TransactionRepo>,
        Arc<dyn TransactionTemplateRepo>,
    ),
) {
    let (user_repo, transaction_repo, _template_repo) = repos;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Misc".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from(10),
            HashSet::new(),
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from(15),
            HashSet::from(["loan".to_string()]),
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("2022-08-02").unwrap(),
            Decimal::from(20),
            HashSet::new(),
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

    test_user.delete().await
}

#[instrument(skip(repos))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_category(
    _tracing_setup: &(),
    repos: (
        Arc<dyn UserRepo>,
        Arc<dyn TransactionRepo>,
        Arc<dyn TransactionTemplateRepo>,
    ),
) {
    let (user_repo, transaction_repo, _template_repo) = repos;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Loan".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from(10),
            HashSet::new(),
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from(15),
            HashSet::from(["loan".to_string()]),
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

    test_user.delete().await
}

#[instrument(skip(repos))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_transactee(
    _tracing_setup: &(),
    repos: (
        Arc<dyn UserRepo>,
        Arc<dyn TransactionRepo>,
        Arc<dyn TransactionTemplateRepo>,
    ),
) {
    let (user_repo, transaction_repo, _template_repo) = repos;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Loan".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from(10),
            HashSet::new(),
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from(15),
            HashSet::from(["loan".to_string()]),
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

    test_user.delete().await
}

#[instrument(skip(repos))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_from(
    _tracing_setup: &(),
    repos: (
        Arc<dyn UserRepo>,
        Arc<dyn TransactionRepo>,
        Arc<dyn TransactionTemplateRepo>,
    ),
) {
    let (user_repo, transaction_repo, _template_repo) = repos;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Loan".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from(10),
            HashSet::new(),
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from(15),
            HashSet::from(["loan".to_string()]),
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

    test_user.delete().await
}

#[instrument(skip(repos))]
#[rstest]
#[actix_rt::test]
async fn test_get_transactions_filter_until(
    _tracing_setup: &(),
    repos: (
        Arc<dyn UserRepo>,
        Arc<dyn TransactionRepo>,
        Arc<dyn TransactionTemplateRepo>,
    ),
) {
    let (user_repo, transaction_repo, _template_repo) = repos;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transactions = vec![
        NewTransaction::new(
            "Loan".to_string(),
            Some("Alice".to_string()),
            None,
            NaiveDate::from_str("2021-10-11").unwrap(),
            Decimal::from(10),
            HashSet::new(),
        ),
        NewTransaction::new(
            "Misc".to_string(),
            Some("Bob".to_string()),
            None,
            NaiveDate::from_str("1900-10-11").unwrap(),
            Decimal::from(15),
            HashSet::from(["loan".to_string()]),
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

    test_user.delete().await
}
