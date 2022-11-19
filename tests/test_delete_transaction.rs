use std::str::FromStr;

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
async fn test_delete_transaction(_tracing_setup: &(), #[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = build_repos(repo_type).await;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let new_transaction = NewTransaction::new(
        "Misc".to_string(),
        Some("Bob".to_string()),
        None,
        NaiveDate::from_str("2021-06-09").unwrap(),
        Decimal::from_str("5.10").unwrap(),
        vec!["Monthly".to_string()],
    );
    let transaction: Transaction = create_transaction!(&service, new_transaction);

    let request = TestRequest::delete()
        .uri(format!("/transactions/{}", transaction.id).as_str())
        .to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let deleted_transaction = test::read_body_json(response).await;
    assert_eq!(transaction, deleted_transaction);

    test_user.delete().await
}

#[instrument]
#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[actix_rt::test]
async fn test_delete_invalid_transaction(_tracing_setup: &(), #[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = build_repos(repo_type).await;
    let test_user = TestUser::new(user_repo).await;
    let app = build_app!(transaction_repo, test_user.user_id.clone());
    let service = test::init_service(app).await;

    let request = TestRequest::delete()
        .uri(format!("/transactions/{}", 0).as_str()) // non-existent transaction ID
        .to_request();
    let response = test::call_service(&service, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    test_user.delete().await
}
