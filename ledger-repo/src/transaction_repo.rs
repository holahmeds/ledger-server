use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use thiserror::Error;

pub struct PageOptions {
    pub offset: i64,
    pub limit: i64,
}

#[async_trait]
pub trait TransactionRepo: Sync + Send {
    async fn get_transaction(
        &self,
        user: String,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError>;

    async fn get_all_transactions(
        &self,
        user: String,
        from: Option<NaiveDate>,
        until: Option<NaiveDate>,
        category: Option<String>,
        transactee: Option<String>,
        page_options: Option<PageOptions>,
    ) -> Result<Vec<Transaction>, TransactionRepoError>;

    async fn create_new_transaction(
        &self,
        user: String,
        new_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError>;

    async fn update_transaction(
        &self,
        user: String,
        transaction_id: i32,
        updated_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError>;

    async fn delete_transaction(
        &self,
        user: String,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError>;

    async fn get_all_categories(&self, user: String) -> Result<Vec<String>, TransactionRepoError>;

    async fn get_all_tags(&self, user: String) -> Result<Vec<String>, TransactionRepoError>;

    async fn get_all_transactees(&self, user: String) -> Result<Vec<String>, TransactionRepoError>;

    async fn get_balance(&self, user: String) -> Result<Decimal, TransactionRepoError>;
}

#[derive(Error, Debug)]
pub enum TransactionRepoError {
    #[error("Transaction with id {0} not found")]
    TransactionNotFound(i32),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Transaction {
    pub id: i32,
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub tags: Vec<String>,
}

impl Transaction {
    pub const fn new(
        id: i32,
        category: String,
        transactee: Option<String>,
        note: Option<String>,
        date: NaiveDate,
        amount: Decimal,
        tags: Vec<String>,
    ) -> Transaction {
        Transaction {
            id,
            category,
            transactee,
            note,
            date,
            amount,
            tags,
        }
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let date_ordering = self.date.partial_cmp(&other.date);
        if let Some(Equal) = date_ordering {
            self.id.partial_cmp(&other.id)
        } else {
            date_ordering
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewTransaction {
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub tags: Vec<String>,
}

impl NewTransaction {
    pub const fn new(
        category: String,
        transactee: Option<String>,
        note: Option<String>,
        date: NaiveDate,
        amount: Decimal,
        tags: Vec<String>,
    ) -> NewTransaction {
        NewTransaction {
            category,
            transactee,
            note,
            date,
            amount,
            tags,
        }
    }
}
