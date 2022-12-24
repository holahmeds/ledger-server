extern crate futures_util;
extern crate rstest;
extern crate serde_json;

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
use utils::tracing_setup;
use utils::TestUser;
use utils::{build_repos, RepoType};

#[macro_use]
mod utils;

#[instrument]
#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[actix_rt::test]
async fn test_create_api_response(_tracing_setup: &(), #[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = build_repos(repo_type).await;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transaction = NewTransaction::new(
        "Misc".to_string(),
        Some("Alice".to_string()),
        None,
        NaiveDate::from_str("2021-07-01").unwrap(),
        Decimal::from(20),
        vec![],
    );
    let response_transaction: Transaction = create_transaction!(&service, new_transaction);
    assert_eq!(new_transaction.category, response_transaction.category);
    assert_eq!(new_transaction.transactee, response_transaction.transactee);
    assert_eq!(new_transaction.category, response_transaction.category);
    assert_eq!(new_transaction.category, response_transaction.category);

    test_user.delete().await
}
