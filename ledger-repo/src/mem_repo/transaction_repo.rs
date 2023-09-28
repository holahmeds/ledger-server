use crate::transaction_repo::TransactionRepoError::TransactionNotFound;
use crate::transaction_repo::{
    Filter, MonthlyTotal, NewTransaction, PageOptions, Transaction, TransactionRepo,
    TransactionRepoError,
};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

struct State {
    transactions: HashMap<i32, Transaction>,
    user_transactions: HashMap<String, HashSet<i32>>,
    next_id: i32,
}

pub struct MemTransactionRepo {
    state: RwLock<State>,
}

impl MemTransactionRepo {
    pub fn new() -> MemTransactionRepo {
        let state = State {
            transactions: HashMap::new(),
            user_transactions: HashMap::new(),
            next_id: 0,
        };
        MemTransactionRepo {
            state: RwLock::new(state),
        }
    }

    fn read_lock(&self) -> Result<RwLockReadGuard<State>, anyhow::Error> {
        self.state
            .read()
            .map_err(|_| anyhow!("Unable to acquire lock"))
    }

    fn write_lock(&self) -> Result<RwLockWriteGuard<State>, anyhow::Error> {
        self.state
            .write()
            .map_err(|_| anyhow!("Unable to acquire lock"))
    }
}

#[async_trait]
impl TransactionRepo for MemTransactionRepo {
    async fn get_transaction(
        &self,
        user: &str,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError> {
        let read_guard = self.read_lock()?;

        let Some(transaction_ids) = read_guard.user_transactions.get(user) else {
            return Err(TransactionNotFound(transaction_id));
        };
        if !transaction_ids.contains(&transaction_id) {
            return Err(TransactionNotFound(transaction_id));
        }

        let transaction = read_guard
            .transactions
            .get(&transaction_id)
            .expect("transactions should contain same ids as user_transactions")
            .clone();
        Ok(transaction)
    }

    async fn get_all_transactions(
        &self,
        user: &str,
        filter: Filter,
        page_options: Option<PageOptions>,
    ) -> Result<Vec<Transaction>, TransactionRepoError> {
        let read_guard = self.read_lock()?;

        let Some(transaction_ids) = read_guard.user_transactions.get(user) else {
            return Ok(Vec::new());
        };

        let mut transactions: Vec<Transaction> = transaction_ids
            .iter()
            .map(|id| {
                read_guard
                    .transactions
                    .get(id)
                    .expect("transactions should have all the ids from user_transactions")
            })
            .cloned()
            .collect();
        transactions.sort_by(|a, b| b.cmp(a));

        let mut transactions: Box<dyn Iterator<Item = Transaction>> =
            Box::new(transactions.into_iter());
        if let Some(from) = filter.from {
            transactions = Box::new(transactions.filter(move |t| t.date >= from));
        }
        if let Some(until) = filter.until {
            transactions = Box::new(transactions.filter(move |t| t.date <= until));
        }
        if let Some(category) = filter.category {
            transactions = Box::new(transactions.filter(move |t| t.category == category));
        }
        if let Some(transactee) = filter.transactee {
            transactions = Box::new(transactions.filter(move |t| {
                if let Some(tr) = &t.transactee {
                    tr == &transactee
                } else {
                    false
                }
            }));
        }

        if let Some(page_options) = page_options {
            transactions = Box::new(
                transactions
                    .skip(page_options.offset as usize)
                    .take(page_options.limit as usize),
            );
        }

        Ok(transactions.collect())
    }

    async fn create_new_transaction(
        &self,
        user: &str,
        new_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError> {
        let mut write_guard = self.write_lock()?;

        let id = write_guard.next_id;
        write_guard.next_id += 1;

        let transaction = new_transaction.to_transaction(id);

        write_guard.transactions.insert(id, transaction.clone());
        write_guard
            .user_transactions
            .entry(user.to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        Ok(transaction)
    }

    async fn update_transaction(
        &self,
        user: &str,
        transaction_id: i32,
        updated_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError> {
        let mut write_guard = self.write_lock()?;

        let Some(transaction_ids) = write_guard.user_transactions.get(user) else {
            return Err(TransactionNotFound(transaction_id));
        };
        if !transaction_ids.contains(&transaction_id) {
            return Err(TransactionNotFound(transaction_id));
        };

        let entry = write_guard.transactions.entry(transaction_id);
        if let Entry::Occupied(mut e) = entry {
            let transaction = updated_transaction.to_transaction(transaction_id);
            e.insert(transaction.clone());
            Ok(transaction)
        } else {
            Err(TransactionNotFound(transaction_id))
        }
    }

    async fn delete_transaction(
        &self,
        user: &str,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError> {
        let mut write_guard = self.write_lock()?;

        if let Some(t) = write_guard.transactions.remove(&transaction_id) {
            write_guard
                .user_transactions
                .get_mut(user)
                .expect("ids in transactions should be present in user_transactions")
                .remove(&transaction_id);
            Ok(t)
        } else {
            Err(TransactionNotFound(transaction_id))
        }
    }

    async fn get_monthly_totals(
        &self,
        user: &str,
        filter: Filter,
    ) -> Result<Vec<MonthlyTotal>, TransactionRepoError> {
        let transactions = self.get_all_transactions(user, filter, None).await?;

        let mut monthly_totals = HashMap::new();
        for t in transactions {
            let month = NaiveDate::from_ymd_opt(t.date.year(), t.date.month(), 1)
                .expect("Transaction dates should be valid");
            let entry = monthly_totals
                .entry(month)
                .or_insert_with(|| MonthlyTotal::new(month, Decimal::ZERO, Decimal::ZERO));
            if t.amount > Decimal::ZERO {
                entry.income += t.amount
            } else {
                entry.expense -= t.amount
            }
        }

        let mut monthly_totals: Vec<MonthlyTotal> = monthly_totals.into_values().collect();
        monthly_totals.sort_by(|a, b| b.month.cmp(&a.month));

        Ok(monthly_totals)
    }

    async fn get_all_categories(&self, user: &str) -> Result<Vec<String>, TransactionRepoError> {
        let categories: HashSet<String> = self
            .get_all_transactions(user, Filter::NONE, None)
            .await?
            .into_iter()
            .map(|t| t.category)
            .collect();
        Ok(categories.into_iter().collect())
    }

    async fn get_all_tags(&self, user: &str) -> Result<Vec<String>, TransactionRepoError> {
        let tags: HashSet<String> = self
            .get_all_transactions(user, Filter::NONE, None)
            .await?
            .into_iter()
            .flat_map(|t| t.tags)
            .collect();
        Ok(tags.into_iter().collect())
    }

    async fn get_all_transactees(
        &self,
        user: &str,
        category: Option<String>,
    ) -> Result<Vec<String>, TransactionRepoError> {
        let mut transactee_counts = HashMap::new();

        let transactions = self.get_all_transactions(user, Filter::NONE, None).await?;
        for x in transactions {
            let Some(transactee) = x.transactee else {
                continue;
            };

            let count = transactee_counts.entry(transactee).or_insert(0);

            if let Some(category) = &category {
                if &x.category == category {
                    *count += 1;
                }
            } else {
                *count += 1;
            }
        }

        let mut transactees: Vec<String> = transactee_counts.keys().cloned().collect();
        transactees.sort_by(|a, b| transactee_counts.get(b).cmp(&transactee_counts.get(a)));

        Ok(transactees)
    }

    async fn get_balance(&self, user: &str) -> Result<Decimal, TransactionRepoError> {
        let sum = self
            .get_all_transactions(user, Filter::NONE, None)
            .await?
            .into_iter()
            .map(|t| t.amount)
            .sum::<Decimal>();
        Ok(sum)
    }
}
