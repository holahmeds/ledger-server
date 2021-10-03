extern crate futures_util;
extern crate serde_json;

use std::str::FromStr;

use actix_web::App;
use actix_web::test;
use actix_web::test::TestRequest;
use actix_web::web;
use chrono::NaiveDate;
use rstest::rstest;
use rust_decimal::Decimal;
use tracing::instrument;

use ledger::{DbPool, transaction_handlers};
use ledger::models::{NewTransaction, Transaction};
use utils::database_pool;
use utils::map_body;

mod utils;

#[instrument(skip(database_pool))]
#[rstest]
#[actix_rt::test]
async fn test_get_all_transactions(database_pool: DbPool) {
    let app = App::new().data(database_pool.clone()).service(
        web::scope("/")
            .service(transaction_handlers::get_all_transactions)
            .service(transaction_handlers::create_new_transaction)
            .service(transaction_handlers::delete_transaction),
    );
    let mut service = test::init_service(app).await;

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

    let mut inserted_transactions = vec![];
    for t in new_transactions {
        let request = TestRequest::post().set_json(&t).to_request();
        let mut response = test::call_service(&mut service, request).await;
        let transaction = map_body::<Transaction>(&mut response).await;
        inserted_transactions.push(transaction);
    }

    let request = TestRequest::get().to_request();
    let mut response = test::call_service(&mut service, request).await;

    let transactions = map_body::<Vec<Transaction>>(&mut response).await;

    assert!(response.status().is_success());

    assert_eq!(inserted_transactions, transactions);

    for t in inserted_transactions {
        let delete_request = TestRequest::delete()
            .uri(format!("/{}", t.id).as_str())
            .to_request();
        test::call_service(&mut service, delete_request).await;
    }
}
