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
use ledger::user::UserId;
use ledger::DbPool;
use utils::database_pool;

#[macro_use]
mod utils;

#[instrument(skip(database_pool))]
#[rstest]
#[actix_rt::test]
async fn test_get_all_transactions(database_pool: &DbPool) {
    let user_id: UserId = "test-user".into();
    utils::create_user(database_pool, &user_id);

    let app = build_app!(database_pool, user_id);
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
        let request = TestRequest::post()
            .uri("/transactions")
            .set_json(&t)
            .to_request();
        let response = test::call_service(&service, request).await;
        let transaction = test::read_body_json(response).await;
        inserted_transactions.push(transaction);
    }

    let request = TestRequest::get().uri("/transactions").to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let transactions: Vec<Transaction> = test::read_body_json(response).await;
    assert_eq!(inserted_transactions, transactions);

    for t in inserted_transactions {
        let delete_request = TestRequest::delete()
            .uri(format!("/transactions/{}", t.id).as_str())
            .to_request();
        test::call_service(&service, delete_request).await;
    }

    utils::delete_user(database_pool, &user_id);
}
