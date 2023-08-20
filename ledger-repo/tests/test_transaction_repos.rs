mod transaction_utils;
mod utils;

use crate::transaction_utils::{
    generate_new_transaction, generate_new_transaction_with_amount,
    generate_new_transaction_with_category, generate_new_transaction_with_date,
    generate_new_transaction_with_date_and_amount, generate_new_transaction_with_tags,
    generate_new_transaction_with_transactee,
};
use chrono::NaiveDate;
use ledger_repo::transaction_repo::{MonthlyTotal, NewTransaction, PageOptions, Transaction};
use ledger_repo::user_repo::{User, UserRepo};
use rstest::rstest;
use rust_decimal::Decimal;
use std::collections::btree_set::BTreeSet;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;
use utils::RepoType;
use uuid::Uuid;

pub struct TestUser {
    pub id: String,
    repo: Arc<dyn UserRepo>,
}

impl TestUser {
    pub async fn new(user_repo: &Arc<dyn UserRepo>) -> TestUser {
        let user_id = "test-user-".to_owned() + &Uuid::new_v4().to_string();
        let user = User::new(user_id.clone(), "not a real hash".to_owned());
        user_repo.create_user(user).await.unwrap();
        info!(%user_id, "Created user");
        TestUser {
            id: user_id,
            repo: user_repo.clone(),
        }
    }

    pub async fn delete(&self) {
        self.repo.delete_user(&self.id).await.unwrap()
    }
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_create_and_get_transactions(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let user = TestUser::new(&user_repo).await;

    let new_transaction = generate_new_transaction();
    let transaction_id = transaction_repo
        .create_new_transaction(&user.id, new_transaction.clone())
        .await
        .unwrap()
        .id;

    let stored_transaction = transaction_repo
        .get_transaction(&user.id, transaction_id)
        .await
        .unwrap();
    assert_eq!(stored_transaction.category, new_transaction.category);
    assert_eq!(stored_transaction.transactee, new_transaction.transactee);
    assert_eq!(stored_transaction.note, new_transaction.note);
    assert_eq!(stored_transaction.date, new_transaction.date);
    assert_eq!(stored_transaction.amount, new_transaction.amount);
    assert_eq!(stored_transaction.tags, new_transaction.tags);

    user.delete().await;
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_invalid_transactions(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let user = TestUser::new(&user_repo).await;

    let get_result = transaction_repo.get_transaction(&user.id, 1234).await;
    assert!(get_result.is_err());
    // TODO
    // assert_eq!(
    //     TransactionRepoError::TransactionNotFound(1234),
    //     get_result.unwrap_err()
    // );

    user.delete().await;
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_invalid_user(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let user1 = TestUser::new(&user_repo).await;
    let user2 = TestUser::new(&user_repo).await;

    let new_transaction = generate_new_transaction();
    let transaction_id = transaction_repo
        .create_new_transaction(&user1.id, new_transaction.clone())
        .await
        .unwrap()
        .id;

    let result = transaction_repo
        .get_transaction(&user2.id, transaction_id)
        .await;
    assert!(result.is_err());
    // TODO
    // assert_eq!(result.unwrap_err(), TransactionNotFound(transaction_id));

    user1.delete().await;
    user2.delete().await;
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_delete_transaction(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let user = TestUser::new(&user_repo).await;

    let new_transaction = generate_new_transaction();
    let transaction_id = transaction_repo
        .create_new_transaction(&user.id, new_transaction.clone())
        .await
        .unwrap()
        .id;

    let delete_result = transaction_repo
        .delete_transaction(&user.id, transaction_id)
        .await;
    assert!(delete_result.is_ok());

    let result = transaction_repo
        .get_transaction(&user.id, transaction_id)
        .await;
    assert!(result.is_err());
    // TODO
    // assert_eq!(
    //     TransactionRepoError::TransactionNotFound(transaction_id),
    //     result.unwrap_err()
    // );

    user.delete().await;
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_delete_invalid_transaction(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let user = TestUser::new(&user_repo).await;

    let delete_result = transaction_repo.delete_transaction(&user.id, 1234).await;
    assert!(delete_result.is_err());
    // TODO
    // assert_eq!(
    //     TransactionRepoError::TransactionNotFound(1234),
    //     delete_result.unwrap_err()
    // );

    user.delete().await;
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_all_transactions(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![generate_new_transaction(), generate_new_transaction()];

    let mut inserted_transactions: BTreeSet<Transaction> = BTreeSet::new();
    for t in new_transactions {
        let transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
        inserted_transactions.insert(transaction);
    }

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(&test_user.id, None, None, None, None, None)
        .await
        .unwrap();
    let transactions = BTreeSet::from_iter(transactions);
    assert_eq!(inserted_transactions, transactions);

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_all_transactions_empty(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(&test_user.id, None, None, None, None, None)
        .await
        .unwrap();
    assert!(transactions.is_empty());

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_transactions_sorted(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![
        generate_new_transaction(),
        generate_new_transaction(),
        generate_new_transaction(),
        generate_new_transaction_with_date(NaiveDate::from_str("2022-12-31").unwrap()),
        generate_new_transaction_with_date(NaiveDate::from_str("2022-12-31").unwrap()),
    ];

    for t in new_transactions {
        let _transaction: Transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
    }

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(&test_user.id, None, None, None, None, None)
        .await
        .unwrap();
    assert!(
        transactions.windows(2).all(|w| w[0] >= w[1]),
        "transactions not sorted"
    );

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_transactions_filter_category(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![
        generate_new_transaction_with_category("Loan".to_owned()),
        generate_new_transaction_with_category("Misc".to_owned()),
    ];

    for t in new_transactions {
        let _transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
    }

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(
            &test_user.id,
            None,
            None,
            Some("Loan".to_owned()),
            None,
            None,
        )
        .await
        .unwrap();
    assert!(transactions.iter().all(|t| t.category == "Loan"));

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_transactions_filter_transactee(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![
        generate_new_transaction_with_transactee("Alice".to_string()),
        generate_new_transaction_with_transactee("Bob".to_string()),
    ];

    for t in new_transactions {
        let _transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
    }

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(
            &test_user.id,
            None,
            None,
            None,
            Some("Alice".to_owned()),
            None,
        )
        .await
        .unwrap();
    assert!(transactions
        .iter()
        .all(|t| t.transactee == Some("Alice".to_owned())));

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_transactions_filter_from(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![
        generate_new_transaction_with_date(NaiveDate::from_str("2021-10-11").unwrap()),
        generate_new_transaction_with_date(NaiveDate::from_str("1900-10-11").unwrap()),
    ];

    for t in new_transactions {
        let _transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
    }

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(
            &test_user.id,
            Some(NaiveDate::from_str("2021-01-01").unwrap()),
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    assert!(transactions
        .iter()
        .all(|t| t.date > NaiveDate::from_str("2021-01-01").unwrap()));

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_transactions_filter_until(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![
        generate_new_transaction_with_date(NaiveDate::from_str("2021-10-11").unwrap()),
        generate_new_transaction_with_date(NaiveDate::from_str("1900-10-11").unwrap()),
    ];

    for t in new_transactions {
        let _transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
    }

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(
            &test_user.id,
            None,
            Some(NaiveDate::from_str("2021-01-01").unwrap()),
            None,
            None,
            None,
        )
        .await
        .unwrap();
    assert!(transactions
        .iter()
        .all(|t| t.date < NaiveDate::from_str("2021-01-01").unwrap()));

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_transactions_pagination(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![
        generate_new_transaction_with_date(NaiveDate::from_str("2022-08-02").unwrap()),
        generate_new_transaction_with_date(NaiveDate::from_str("2021-10-11").unwrap()),
        generate_new_transaction_with_date(NaiveDate::from_str("1900-10-11").unwrap()),
    ];

    let mut inserted_transactions = vec![];
    for t in new_transactions {
        let transaction: Transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
        inserted_transactions.push(transaction);
    }

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(
            &test_user.id,
            None,
            None,
            None,
            None,
            Some(PageOptions::new(1, 2)),
        )
        .await
        .unwrap();
    assert_eq!(2, transactions.len());
    assert_eq!(transactions.get(0), inserted_transactions.get(1));
    assert_eq!(transactions.get(1), inserted_transactions.get(2));

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_update_transaction(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transaction = generate_new_transaction();
    let transaction: Transaction = transaction_repo
        .create_new_transaction(&test_user.id, new_transaction.clone())
        .await
        .unwrap();

    let update = NewTransaction::new(
        new_transaction.category,
        Some("Alice".to_string()),
        new_transaction.note,
        new_transaction.date,
        Decimal::from(105),
        HashSet::new(),
    );

    let updated_transaction: Transaction = transaction_repo
        .update_transaction(&test_user.id, transaction.id, update.clone())
        .await
        .unwrap();
    assert_eq!(transaction.id, updated_transaction.id);
    assert_ne!(transaction, updated_transaction);
    assert_eq!(updated_transaction.transactee, update.transactee);

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_update_tags(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transaction = generate_new_transaction();
    let transaction: Transaction = transaction_repo
        .create_new_transaction(&test_user.id, new_transaction.clone())
        .await
        .unwrap();

    let update = NewTransaction::new(
        new_transaction.category,
        Some("Alice".to_string()),
        new_transaction.note,
        new_transaction.date,
        Decimal::from(105),
        HashSet::from(["tag2".to_string(), "tag3".to_string()]),
    );
    let updated_transaction: Transaction = transaction_repo
        .update_transaction(&test_user.id, transaction.id, update.clone())
        .await
        .unwrap();

    assert_eq!(transaction.id, updated_transaction.id);
    assert_ne!(transaction, updated_transaction);
    assert_eq!(updated_transaction.transactee, update.transactee);
    assert_eq!(updated_transaction.tags, update.tags);

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_update_invalid_transaction(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let update = generate_new_transaction();

    let result = transaction_repo
        .update_transaction(&test_user.id, 1234, update)
        .await;
    assert!(result.is_err());
    // TODO
    // assert_eq!(
    //     TransactionRepoError::TransactionNotFound(1234),
    //     result.unwrap_err()
    // );

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_update_invalid_user(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let user1 = TestUser::new(&user_repo).await;
    let user2 = TestUser::new(&user_repo).await;

    let new_transaction = generate_new_transaction();
    let transaction = transaction_repo
        .create_new_transaction(&user1.id, new_transaction)
        .await
        .unwrap();

    let update = generate_new_transaction();
    let result = transaction_repo
        .update_transaction(&user2.id, transaction.id, update)
        .await;
    assert!(result.is_err());
    // TODO
    // assert_eq!(
    //     TransactionRepoError::TransactionNotFound(1234),
    //     result.unwrap_err()
    // );

    user1.delete().await;
    user2.delete().await;
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_monthly_totals(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![
        generate_new_transaction_with_date_and_amount(
            NaiveDate::from_str("2022-12-02").unwrap(),
            Decimal::from(-20),
        ),
        generate_new_transaction_with_date_and_amount(
            NaiveDate::from_str("2022-12-11").unwrap(),
            Decimal::from(10),
        ),
        generate_new_transaction_with_date_and_amount(
            NaiveDate::from_str("2022-12-11").unwrap(),
            Decimal::from(15),
        ),
        generate_new_transaction_with_date_and_amount(
            NaiveDate::from_str("2022-11-11").unwrap(),
            Decimal::from(30),
        ),
    ];

    for t in new_transactions {
        let _transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
    }

    let monthly_totals = transaction_repo
        .get_monthly_totals(&test_user.id)
        .await
        .unwrap();
    assert_eq!(
        monthly_totals,
        vec![
            MonthlyTotal::new(
                NaiveDate::from_str("2022-12-1").unwrap(),
                Decimal::from(25),
                Decimal::from(20),
            ),
            MonthlyTotal::new(
                NaiveDate::from_str("2022-11-1").unwrap(),
                Decimal::from(30),
                Decimal::ZERO,
            ),
        ]
    );

    test_user.delete().await;
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_categories(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![
        generate_new_transaction_with_category("Misc".to_string()),
        generate_new_transaction_with_category("Loan".to_string()),
        generate_new_transaction_with_category("Misc".to_string()),
    ];

    for t in new_transactions {
        let _transaction: Transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
    }

    let categories = transaction_repo
        .get_all_categories(&test_user.id)
        .await
        .unwrap();
    assert_eq!(
        HashSet::from(["Loan".to_owned(), "Misc".to_owned()]),
        HashSet::from_iter(categories.into_iter())
    );

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_tags(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![
        generate_new_transaction_with_tags(HashSet::from(["tag1".to_string(), "tag2".to_string()])),
        generate_new_transaction_with_tags(HashSet::from(["tag2".to_string(), "tag3".to_string()])),
    ];

    for t in new_transactions {
        let _transaction: Transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
    }

    let tags = transaction_repo.get_all_tags(&test_user.id).await.unwrap();
    assert_eq!(
        HashSet::from(["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]),
        HashSet::from_iter(tags.into_iter())
    );

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_transactees(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![
        generate_new_transaction_with_transactee("Bob".to_string()),
        generate_new_transaction_with_transactee("Alice".to_string()),
        generate_new_transaction_with_transactee("Bob".to_string()),
    ];

    for t in new_transactions {
        let _transaction: Transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
    }

    let transactees = transaction_repo
        .get_all_transactees(&test_user.id)
        .await
        .unwrap();
    assert_eq!(
        HashSet::from(["Alice".to_owned(), "Bob".to_owned()]),
        HashSet::from_iter(transactees.into_iter())
    );

    test_user.delete().await
}

#[rstest]
#[case::diesel(RepoType::Diesel)]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_balance(#[case] repo_type: RepoType) {
    let (transaction_repo, user_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let new_transactions = vec![
        generate_new_transaction_with_amount(Decimal::from(20)),
        generate_new_transaction_with_amount(Decimal::from(-10)),
        generate_new_transaction_with_amount(Decimal::from(15)),
    ];

    for t in new_transactions {
        let _transaction: Transaction = transaction_repo
            .create_new_transaction(&test_user.id, t)
            .await
            .unwrap();
    }

    let balance = transaction_repo.get_balance(&test_user.id).await.unwrap();
    assert_eq!(Decimal::from(25), balance);

    test_user.delete().await
}
