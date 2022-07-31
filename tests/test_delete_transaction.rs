use std::str::FromStr;

use actix_web::http::StatusCode;
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
async fn test_delete_transaction(database_pool: &DbPool) {
    let user_id: UserId = "test-user".into();
    utils::create_user(database_pool, &user_id);

    let app = build_app!(database_pool, user_id);
    let service = test::init_service(app).await;

    let new_transaction = NewTransaction::new(
        "Misc".to_string(),
        Some("Bob".to_string()),
        None,
        NaiveDate::from_str("2021-06-09").unwrap(),
        Decimal::from_str("5.10").unwrap(),
        vec!["Monthly".to_string()],
    );
    let transaction: Transaction = {
        let request = TestRequest::post()
            .uri("/transactions")
            .set_json(&new_transaction)
            .to_request();
        let response = test::call_service(&service, request).await;
        assert!(
            response.status().is_success(),
            "Failed to create transaction. Response status {}",
            response.status()
        );
        test::read_body_json(response).await
    };

    let request = TestRequest::delete()
        .uri(format!("/transactions/{}", transaction.id).as_str())
        .to_request();
    let response = test::call_service(&service, request).await;
    assert!(response.status().is_success());

    let deleted_transaction = test::read_body_json(response).await;
    assert_eq!(transaction, deleted_transaction);

    utils::delete_user(database_pool, &user_id);
}

#[instrument(skip(database_pool))]
#[rstest]
#[actix_rt::test]
async fn test_delete_invalid_transaction(database_pool: &DbPool) {
    let user_id: UserId = "test-user2".into();
    utils::create_user(database_pool, &user_id);

    let state = Data::new(database_pool.clone());
    let app = App::new()
        .app_data(state)
        .service(web::scope("/").service(handlers::delete_transaction))
        .wrap(MockAuthentication {
            user_id: user_id.clone(),
        });
    let service = test::init_service(app).await;

    let request = TestRequest::delete()
        .uri(format!("/{}", 0).as_str()) // non-existent transaction ID
        .to_request();
    let response = test::call_service(&service, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    utils::delete_user(database_pool, &user_id);
}
