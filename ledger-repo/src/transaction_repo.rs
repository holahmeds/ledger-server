use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug)]
pub struct PageOptions {
    pub offset: i64,
    pub limit: i64,
}

#[derive(Debug)]
pub struct Filter {
    pub from: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub category: Option<String>,
    pub transactee: Option<String>,
}

impl Filter {
    pub const NONE: Filter = Filter {
        from: None,
        until: None,
        category: None,
        transactee: None,
    };

    pub fn new(
        from: Option<NaiveDate>,
        until: Option<NaiveDate>,
        category: Option<String>,
        transactee: Option<String>,
    ) -> Filter {
        Filter {
            from,
            until,
            category,
            transactee,
        }
    }
}

impl PageOptions {
    pub fn new(offset: i64, limit: i64) -> PageOptions {
        PageOptions { offset, limit }
    }
}

#[async_trait]
pub trait TransactionRepo: Sync + Send {
    async fn get_transaction(
        &self,
        user: &str,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError>;

    async fn get_all_transactions(
        &self,
        user: &str,
        filter: Filter,
        page_options: Option<PageOptions>,
    ) -> Result<Vec<Transaction>, TransactionRepoError>;

    async fn create_new_transaction(
        &self,
        user: &str,
        new_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError>;

    async fn update_transaction(
        &self,
        user: &str,
        transaction_id: i32,
        updated_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError>;

    async fn delete_transaction(
        &self,
        user: &str,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError>;

    async fn get_monthly_totals(
        &self,
        user: &str,
        filter: Filter,
    ) -> Result<Vec<MonthlyTotal>, TransactionRepoError>;

    async fn get_all_categories(&self, user: &str) -> Result<Vec<String>, TransactionRepoError>;

    async fn get_all_tags(&self, user: &str) -> Result<Vec<String>, TransactionRepoError>;

    async fn get_all_transactees(
        &self,
        user: &str,
        category: Option<String>,
    ) -> Result<Vec<String>, TransactionRepoError>;

    async fn get_balance(&self, user: &str) -> Result<Decimal, TransactionRepoError>;
}

#[derive(Error, Debug)]
pub enum TransactionRepoError {
    #[error("Transaction with id {0} not found")]
    TransactionNotFound(i32),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Transaction {
    pub id: i32,
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub tags: HashSet<String>,
}

impl Transaction {
    pub const fn new(
        id: i32,
        category: String,
        transactee: Option<String>,
        note: Option<String>,
        date: NaiveDate,
        amount: Decimal,
        tags: HashSet<String>,
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

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        let date_ordering = self.date.cmp(&other.date);
        if date_ordering == Equal {
            self.id.cmp(&other.id)
        } else {
            date_ordering
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTransaction {
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub tags: HashSet<String>,
}

impl NewTransaction {
    pub const fn new(
        category: String,
        transactee: Option<String>,
        note: Option<String>,
        date: NaiveDate,
        amount: Decimal,
        tags: HashSet<String>,
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

    pub fn to_transaction(self, id: i32) -> Transaction {
        Transaction::new(
            id,
            self.category,
            self.transactee,
            self.note,
            self.date,
            self.amount,
            self.tags,
        )
    }
}

#[derive(PartialEq, Debug)]
pub struct MonthlyTotal {
    pub month: NaiveDate,
    pub income: Decimal,
    pub expense: Decimal,
}

impl MonthlyTotal {
    pub fn new(month: NaiveDate, income: Decimal, expense: Decimal) -> MonthlyTotal {
        MonthlyTotal {
            month,
            income,
            expense,
        }
    }
}
