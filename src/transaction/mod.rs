use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;

use crate::user::UserId;
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

    fn from_entry_and_tags(transaction_entry: TransactionEntry, tags: Vec<String>) -> Transaction {
        Transaction {
            id: transaction_entry.id,
            category: transaction_entry.category,
            transactee: transaction_entry.transactee,
            note: transaction_entry.note,
            date: transaction_entry.date,
            amount: transaction_entry.amount,
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

    fn split_tags(self, user_id: UserId) -> (NewTransactionEntry, Vec<String>) {
        let new_transaction_entry = NewTransactionEntry {
            category: self.category,
            transactee: self.transactee,
            note: self.note,
            date: self.date,
            amount: self.amount,
            user_id,
        };
        (new_transaction_entry, self.tags)
    }
}
