extern crate futures_util;
extern crate rstest;
extern crate serde_json;

use std::collections::HashSet;
use std::str::FromStr;

use actix_web::test;
use actix_web::test::TestRequest;
use actix_web::web::Data;
use actix_web::App;
use chrono::NaiveDate;
use rstest::rstest;
use rust_decimal::Decimal;
use tracing::instrument;

use crate::utils::mock::MockAuthentication;
use ledger_repo::transaction_repo::{NewTransaction, Transaction};
use utils::build_repos;
use utils::tracing_setup;
use utils::TestUser;

#[macro_use]
mod utils;

#[instrument]
#[rstest]
#[actix_rt::test]
async fn test_create_api_response(_tracing_setup: &()) {
    let (transaction_repo, user_repo) = build_repos().await;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transaction = NewTransaction::new(
        "Misc".to_string(),
        Some("Alice".to_string()),
        None,
        NaiveDate::from_str("2021-07-01").unwrap(),
        Decimal::from(20),
        HashSet::new(),
    );
    let response_transaction: Transaction = create_transaction!(&service, new_transaction);
    assert_eq!(new_transaction.category, response_transaction.category);
    assert_eq!(new_transaction.transactee, response_transaction.transactee);
    assert_eq!(new_transaction.category, response_transaction.category);
    assert_eq!(new_transaction.category, response_transaction.category);

    test_user.delete().await
}
