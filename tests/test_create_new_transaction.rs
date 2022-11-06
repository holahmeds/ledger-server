extern crate futures_util;
extern crate rstest;
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
use ledger::repo::transaction_repo::{NewTransaction, Transaction, TransactionRepo};
use ledger::repo::user_repo::UserRepo;
use utils::transaction_repo;
use utils::user_repo;
use utils::TestUser;

#[macro_use]
mod utils;

#[instrument(skip(transaction_repo, user_repo))]
#[rstest]
#[actix_rt::test]
async fn test_create_api_response(
    transaction_repo: Arc<dyn TransactionRepo>,
    user_repo: Arc<dyn UserRepo>,
) {
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
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
}
