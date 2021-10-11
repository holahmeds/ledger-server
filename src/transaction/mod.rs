use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use models::NewTransactionEntry;
use models::TransactionEntry;

pub mod handlers;
mod models;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Transaction {
    pub id: i32,
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub transaction_date: NaiveDate,
    pub amount: Decimal,
    pub tags: Vec<String>,
}

impl Transaction {
    pub const fn new(
        id: i32,
        category: String,
        transactee: Option<String>,
        note: Option<String>,
        transaction_date: NaiveDate,
        amount: Decimal,
        tags: Vec<String>,
    ) -> Transaction {
        Transaction {
            id,
            category,
            transactee,
            note,
            transaction_date,
            amount,
            tags,
        }
    }

    fn from_entry_and_tags(transaction_entry: TransactionEntry, tags: Vec<String>) -> Transaction {
        Transaction {
            id: transaction_entry.id,
            category: transaction_entry.category,
            transactee: transaction_entry.transactee,
            note: transaction_entry.note,
            transaction_date: transaction_entry.transaction_date,
            amount: transaction_entry.amount,
            tags,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewTransaction {
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub transaction_date: NaiveDate,
    pub amount: Decimal,
    pub tags: Vec<String>,
}

impl NewTransaction {
    pub const fn new(
        category: String,
        transactee: Option<String>,
        note: Option<String>,
        transaction_date: NaiveDate,
        amount: Decimal,
        tags: Vec<String>,
    ) -> NewTransaction {
        NewTransaction {
            category,
            transactee,
            note,
            transaction_date,
            amount,
            tags,
        }
    }

    fn split_tags(self) -> (NewTransactionEntry, Vec<String>) {
        let new_transaction_entry = NewTransactionEntry {
            category: self.category,
            transactee: self.transactee,
            note: self.note,
            transaction_date: self.transaction_date,
            amount: self.amount,
        };
        (new_transaction_entry, self.tags)
    }
}
