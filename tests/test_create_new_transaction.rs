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

#[macro_use]
mod utils;

#[instrument(skip(database_pool))]
#[rstest]
#[actix_rt::test]
async fn test_create_api_response(database_pool: &DbPool) {
    let user_id: UserId = "test-user".into();
    utils::create_user(database_pool, &user_id);

    let app = build_app!(database_pool, user_id);
    let service = test::init_service(app).await;

    let new_transaction = NewTransaction::new(
        "Misc".to_string(),
        Some("Alice".to_string()),
        None,
        NaiveDate::from_str("2021-07-01").unwrap(),
        Decimal::from_str("20").unwrap(),
        vec![],
    );
    let response_transaction: Transaction = create_transaction!(&service, new_transaction);
    assert_eq!(new_transaction.category, response_transaction.category);
    assert_eq!(new_transaction.transactee, response_transaction.transactee);
    assert_eq!(new_transaction.category, response_transaction.category);
    assert_eq!(new_transaction.category, response_transaction.category);

    delete_transaction!(&service, response_transaction.id);

    utils::delete_user(database_pool, &user_id);
}
