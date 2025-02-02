mod utils;

use chrono::NaiveDate;
use futures::future::try_join_all;
use ledger_repo::transaction_repo::{
    Filter, MonthlyTotal, NewTransaction, PageOptions, Transaction, TransactionRepo,
    TransactionRepoError,
};
use rstest::rstest;
use rust_decimal::Decimal;
use std::collections::btree_set::BTreeSet;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use utils::generator::NewTransactionGenerator;
use utils::test_user::TestUser;
use utils::RepoType;

pub fn generate_new_transaction() -> NewTransaction {
    let mut generator = NewTransactionGenerator::default();
    generator.generate()
}

async fn insert_transactions(
    transaction_repo: &Arc<dyn TransactionRepo>,
    test_user: &TestUser,
    new_transactions: Vec<NewTransaction>,
) -> Result<Vec<Transaction>, TransactionRepoError> {
    let inserted_transactions = new_transactions
        .into_iter()
        .map(|t| transaction_repo.create_new_transaction(&test_user.id, t));
    try_join_all(inserted_transactions).await
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_create_and_get_transactions(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _transaction_template_repo) =
        utils::build_repos(repo_type).await;
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
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_invalid_transactions(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let user = TestUser::new(&user_repo).await;

    let get_result = transaction_repo.get_transaction(&user.id, 1234).await;
    assert!(matches!(
        get_result,
        Err(TransactionRepoError::TransactionNotFound(1234))
    ));

    user.delete().await;
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_invalid_user(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
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
    assert!(matches!(
        result,
        Err(TransactionRepoError::TransactionNotFound(_))
    ));

    user1.delete().await;
    user2.delete().await;
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_delete_transaction(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
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
    assert!(matches!(
        result,
        Err(TransactionRepoError::TransactionNotFound(_))
    ));

    user.delete().await;
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_delete_invalid_transaction(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let user = TestUser::new(&user_repo).await;

    let delete_result = transaction_repo.delete_transaction(&user.id, 1234).await;
    assert!(matches!(
        delete_result,
        Err(TransactionRepoError::TransactionNotFound(_))
    ));

    user.delete().await;
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_all_transactions(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator = NewTransactionGenerator::default();
    let new_transactions = generator.generate_many(2);

    let inserted_transactions =
        insert_transactions(&transaction_repo, &test_user, new_transactions)
            .await
            .unwrap();

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(&test_user.id, Filter::NONE, None)
        .await
        .unwrap();
    assert_eq!(
        BTreeSet::from_iter(inserted_transactions),
        BTreeSet::from_iter(transactions)
    );

    test_user.delete().await
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_all_transactions_empty(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(&test_user.id, Filter::NONE, None)
        .await
        .unwrap();
    assert!(transactions.is_empty());

    test_user.delete().await
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_transactions_sorted(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator = NewTransactionGenerator::default();
    let mut new_transactions = generator.generate_many(3);
    // generate two transactions with the same dates but different IDs
    // this is to make sure that transactions are sorted by dates and then IDs
    generator = generator.with_dates(vec![
        NaiveDate::from_str("2022-12-31").unwrap(),
        NaiveDate::from_str("2022-12-31").unwrap(),
    ]);
    new_transactions.extend(generator.generate_many(2));

    insert_transactions(&transaction_repo, &test_user, new_transactions)
        .await
        .unwrap();

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(&test_user.id, Filter::NONE, None)
        .await
        .unwrap();
    assert!(
        transactions.windows(2).all(|w| w[0] >= w[1]),
        "transactions not sorted"
    );

    test_user.delete().await
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_transactions_filter_category(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator = NewTransactionGenerator::default().with_categories(vec!["Loan", "Misc"]);
    let new_transactions = generator.generate_many(2);

    insert_transactions(&transaction_repo, &test_user, new_transactions)
        .await
        .unwrap();

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(
            &test_user.id,
            Filter::new(None, None, Some("Loan".to_owned()), None),
            None,
        )
        .await
        .unwrap();
    assert!(transactions.iter().all(|t| t.category == "Loan"));

    test_user.delete().await
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_transactions_filter_transactee(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator = NewTransactionGenerator::default().with_transactees(vec!["Alice", "Bob"]);
    let new_transactions = generator.generate_many(2);

    insert_transactions(&transaction_repo, &test_user, new_transactions)
        .await
        .unwrap();

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(
            &test_user.id,
            Filter::new(None, None, None, Some("Alice".to_owned())),
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
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_transactions_filter_from(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator = NewTransactionGenerator::default().with_dates(vec![
        NaiveDate::from_str("2021-10-11").unwrap(),
        NaiveDate::from_str("1900-10-11").unwrap(),
    ]);
    let new_transactions = generator.generate_many(2);

    insert_transactions(&transaction_repo, &test_user, new_transactions)
        .await
        .unwrap();

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(
            &test_user.id,
            Filter::new(
                Some(NaiveDate::from_str("2021-01-01").unwrap()),
                None,
                None,
                None,
            ),
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
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_transactions_filter_until(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator = NewTransactionGenerator::default().with_dates(vec![
        NaiveDate::from_str("2021-10-11").unwrap(),
        NaiveDate::from_str("1900-10-11").unwrap(),
    ]);
    let new_transactions = generator.generate_many(2);

    insert_transactions(&transaction_repo, &test_user, new_transactions)
        .await
        .unwrap();

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(
            &test_user.id,
            Filter::new(
                None,
                Some(NaiveDate::from_str("2021-01-01").unwrap()),
                None,
                None,
            ),
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
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_transactions_pagination(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator = NewTransactionGenerator::default().with_dates(vec![
        NaiveDate::from_str("2022-08-02").unwrap(),
        NaiveDate::from_str("2021-10-11").unwrap(),
        NaiveDate::from_str("1900-10-11").unwrap(),
    ]);
    let new_transactions = generator.generate_many(3);

    let inserted_transactions =
        insert_transactions(&transaction_repo, &test_user, new_transactions)
            .await
            .unwrap();

    let transactions: Vec<Transaction> = transaction_repo
        .get_all_transactions(&test_user.id, Filter::NONE, Some(PageOptions::new(1, 2)))
        .await
        .unwrap();
    assert_eq!(2, transactions.len());
    assert_eq!(transactions.get(0), inserted_transactions.get(1));
    assert_eq!(transactions.get(1), inserted_transactions.get(2));

    test_user.delete().await
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_update_transaction(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
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
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_update_tags(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
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
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_update_invalid_transaction(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let update = generate_new_transaction();

    let result = transaction_repo
        .update_transaction(&test_user.id, 1234, update)
        .await;
    assert!(matches!(
        result,
        Err(TransactionRepoError::TransactionNotFound(_))
    ));

    test_user.delete().await
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_update_invalid_user(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
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
    assert!(matches!(
        result,
        Err(TransactionRepoError::TransactionNotFound(_))
    ));

    user1.delete().await;
    user2.delete().await;
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_monthly_totals(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator = NewTransactionGenerator::default()
        .with_dates(vec![
            NaiveDate::from_str("2022-12-02").unwrap(),
            NaiveDate::from_str("2022-12-11").unwrap(),
            NaiveDate::from_str("2022-12-11").unwrap(),
            NaiveDate::from_str("2022-11-11").unwrap(),
        ])
        .with_amounts(vec![
            Decimal::from(-20),
            Decimal::from(10),
            Decimal::from(15),
            Decimal::from(30),
        ]);
    let new_transactions = generator.generate_many(4);

    insert_transactions(&transaction_repo, &test_user, new_transactions)
        .await
        .unwrap();

    let monthly_totals = transaction_repo
        .get_monthly_totals(&test_user.id, Filter::NONE)
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
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_categories(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator =
        NewTransactionGenerator::default().with_categories(vec!["Misc", "Loan", "Misc"]);
    let new_transactions = generator.generate_many(3);

    insert_transactions(&transaction_repo, &test_user, new_transactions)
        .await
        .unwrap();

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
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_tags(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator = NewTransactionGenerator::default().with_tags(vec![
        HashSet::from(["tag1".to_string(), "tag2".to_string()]),
        HashSet::from(["tag2".to_string(), "tag3".to_string()]),
    ]);
    let new_transactions = generator.generate_many(2);

    insert_transactions(&transaction_repo, &test_user, new_transactions)
        .await
        .unwrap();

    let tags = transaction_repo.get_all_tags(&test_user.id).await.unwrap();
    assert_eq!(
        HashSet::from(["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]),
        HashSet::from_iter(tags.into_iter())
    );

    test_user.delete().await
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_transactees(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator =
        NewTransactionGenerator::default().with_transactees(vec!["Bob", "Alice", "Bob"]);
    let new_transactions = generator.generate_many(3);

    insert_transactions(&transaction_repo, &test_user, new_transactions)
        .await
        .unwrap();

    let transactees = transaction_repo
        .get_all_transactees(&test_user.id, None)
        .await
        .unwrap();
    // Should be sorted by most transactions
    assert_eq!(vec!["Bob".to_owned(), "Alice".to_owned()], transactees);

    test_user.delete().await
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_category_transactees(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator = NewTransactionGenerator::default()
        .with_categories(vec!["Misc", "Groceries", "Misc"])
        .with_transactees(vec!["Bob", "Alice", "Bob"]);
    let new_transactions = generator.generate_many(3);

    insert_transactions(&transaction_repo, &test_user, new_transactions)
        .await
        .unwrap();

    let transactees = transaction_repo
        .get_all_transactees(&test_user.id, Some("Groceries".to_string()))
        .await
        .unwrap();
    // Should be sorted by most transactions of that category
    assert_eq!(vec!["Alice".to_owned(), "Bob".to_owned()], transactees);

    test_user.delete().await
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_balance(#[case] repo_type: RepoType) {
    let (user_repo, transaction_repo, _template_repo) = utils::build_repos(repo_type).await;
    let test_user = TestUser::new(&user_repo).await;

    let mut generator = NewTransactionGenerator::default().with_amounts(vec![
        Decimal::from(20),
        Decimal::from(-10),
        Decimal::from(15),
    ]);
    let new_transactions = generator.generate_many(3);

    insert_transactions(&transaction_repo, &test_user, new_transactions)
        .await
        .unwrap();

    let balance = transaction_repo.get_balance(&test_user.id).await.unwrap();
    assert_eq!(Decimal::from(25), balance);

    test_user.delete().await
}
