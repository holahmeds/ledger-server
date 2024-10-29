use super::schema::{transaction_tags, transactions};
use super::DbPool;
use crate::transaction_repo::{
    Filter, MonthlyTotal, NewTransaction, PageOptions, Transaction, TransactionRepo,
    TransactionRepoError,
};
use actix_web::web;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use diesel::dsl::{sql, sum};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::sql_types::Text;
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use r2d2::PooledConnection;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::instrument;

#[derive(Queryable, Identifiable)]
#[diesel(table_name = transactions)]
struct TransactionEntry {
    id: i32,
    category: String,
    transactee: Option<String>,
    note: Option<String>,
    date: NaiveDate,
    amount: Decimal,
    #[allow(dead_code)]
    user_id: String,
    tags: Vec<Option<String>>,
}

impl From<TransactionEntry> for Transaction {
    fn from(value: TransactionEntry) -> Self {
        let tags = value.tags.into_iter().filter_map(|t| t).collect();
        Transaction::new(
            value.id,
            value.category,
            value.transactee,
            value.note,
            value.date,
            value.amount,
            tags,
        )
    }
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = transactions)]
struct NewTransactionEntry {
    category: String,
    transactee: Option<String>,
    note: Option<String>,
    date: NaiveDate,
    amount: Decimal,
    user_id: String,
    tags: Vec<String>,
}

impl NewTransactionEntry {
    fn from_new_transaction(new_transaction: NewTransaction, user_id: String) -> Self {
        NewTransactionEntry {
            category: new_transaction.category,
            transactee: new_transaction.transactee,
            note: new_transaction.note,
            date: new_transaction.date,
            amount: new_transaction.amount,
            user_id,
            tags: new_transaction.tags.into_iter().collect(),
        }
    }
}

#[derive(Associations, Identifiable, Queryable, Insertable, PartialEq, Eq, Hash)]
#[diesel(primary_key(transaction_id, tag))]
#[diesel(belongs_to(TransactionEntry, foreign_key = transaction_id))]
struct TransactionTag {
    transaction_id: i32,
    tag: String,
}

pub struct DieselTransactionRepo {
    db_pool: DbPool,
}

impl DieselTransactionRepo {
    pub fn new(db_pool: DbPool) -> DieselTransactionRepo {
        DieselTransactionRepo { db_pool }
    }

    async fn block<F, R>(&self, f: F) -> Result<R, TransactionRepoError>
    where
        F: FnOnce(
                &mut PooledConnection<ConnectionManager<PgConnection>>,
            ) -> Result<R, TransactionRepoError>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let pool = self.db_pool.clone();
        web::block(move || {
            let mut db_conn = pool.get().context("Unable to get connection from pool")?;
            f(&mut db_conn)
        })
        .await
        .context("Blocking error")?
    }
}

#[async_trait]
impl TransactionRepo for DieselTransactionRepo {
    #[instrument(skip(self))]
    async fn get_transaction(
        &self,
        user: &str,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError> {
        let user = user.to_owned();
        self.block(move |db_conn| {
            use crate::diesel_repo::schema::transactions::dsl::*;
            use diesel::QueryDsl;

            let transaction_entry: TransactionEntry = transactions
                .find(transaction_id)
                .filter(user_id.eq(user))
                .get_result(db_conn)
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => {
                        TransactionRepoError::TransactionNotFound(transaction_id)
                    }
                    _ => TransactionRepoError::Other(anyhow::Error::new(e).context(format!(
                        "Unable to get transaction {} from database",
                        transaction_id
                    ))),
                })?;

            Ok(transaction_entry.into())
        })
        .await
    }

    #[instrument(skip(self))]
    async fn get_all_transactions(
        &self,
        user: &str,
        filter: Filter,
        page_options: Option<PageOptions>,
    ) -> Result<Vec<Transaction>, TransactionRepoError> {
        let user = user.to_owned();
        self.block(move |db_conn| {
            let mut query = transactions::table
                .filter(transactions::user_id.eq(user))
                .into_boxed();
            if let Some(from) = filter.from {
                query = query.filter(transactions::date.ge(from))
            }
            if let Some(until) = filter.until {
                query = query.filter(transactions::date.le(until))
            }
            if let Some(category) = filter.category {
                query = query.filter(transactions::category.eq(category))
            }
            if let Some(transactee) = filter.transactee {
                query = query.filter(transactions::transactee.eq(transactee))
            }
            if let Some(po) = page_options {
                query = query.offset(po.offset).limit(po.limit)
            }

            let transactions_list = query
                .order((transactions::date.desc(), transactions::id.desc()))
                .load::<TransactionEntry>(db_conn)
                .context("Unable to retrieve transactions")?
                .into_iter()
                .map(|transaction_entry| transaction_entry.into())
                .collect();
            Ok(transactions_list)
        })
        .await
    }

    #[instrument(skip(self, new_transaction))]
    async fn create_new_transaction(
        &self,
        user: &str,
        new_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError> {
        let new_transaction_entry =
            NewTransactionEntry::from_new_transaction(new_transaction, user.to_owned());

        self.block(move |db_conn| {
            let transaction_entry: TransactionEntry = diesel::insert_into(transactions::table)
                .values(new_transaction_entry)
                .get_result(db_conn)
                .context("Unable to insert transaction")?;
            Ok(transaction_entry.into())
        })
        .await
    }

    #[instrument(skip(self, updated_transaction))]
    async fn update_transaction(
        &self,
        user: &str,
        transaction_id: i32,
        updated_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError> {
        let new_transaction_entry =
            NewTransactionEntry::from_new_transaction(updated_transaction, user.to_owned());

        self.block(move |db_conn| {
            let transaction_entry: TransactionEntry = diesel::update(
                transactions::table
                    .find(transaction_id)
                    .filter(transactions::user_id.eq(&new_transaction_entry.user_id)),
            )
                .set(&new_transaction_entry)
                .get_result(db_conn)
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => {
                        TransactionRepoError::TransactionNotFound(transaction_id)
                    }
                    _ => TransactionRepoError::Other(
                        anyhow::Error::new(e)
                            .context(format!("Unable to update transaction {}", transaction_id)),
                    ),
                })?;

            Ok(transaction_entry.into())
        })
        .await
    }

    #[instrument(skip(self))]
    async fn delete_transaction(
        &self,
        user: &str,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError> {
        let user = user.to_owned();
        self.block(move |db_conn| {
            let transaction_entry: TransactionEntry = diesel::delete(
                transactions::table
                    .find(transaction_id)
                    .filter(transactions::user_id.eq(user)),
            )
            .get_result(db_conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    TransactionRepoError::TransactionNotFound(transaction_id)
                }
                _ => TransactionRepoError::Other(
                    anyhow::Error::new(e)
                        .context(format!("Unable to delete transaction {}", transaction_id)),
                ),
            })?;

            Ok(transaction_entry.into())
        })
        .await
    }

    #[instrument(skip(self))]
    async fn get_monthly_totals(
        &self,
        user: &str,
        filter: Filter,
    ) -> Result<Vec<MonthlyTotal>, TransactionRepoError> {
        let user = user.to_owned();
        self.block(move |db_conn| {
            #[derive(Queryable)]
            struct Entry {
                date: NaiveDate,
                amount: Decimal,
            }

            let mut query = transactions::table
                .filter(transactions::user_id.eq(&user))
                .into_boxed();
            if let Some(from) = filter.from {
                query = query.filter(transactions::date.ge(from))
            }
            if let Some(until) = filter.until {
                query = query.filter(transactions::date.le(until))
            }
            if let Some(category) = filter.category {
                query = query.filter(transactions::category.eq(category))
            }
            if let Some(transactee) = filter.transactee {
                query = query.filter(transactions::transactee.eq(transactee))
            }

            let entries: Vec<Entry> = query
                .select((transactions::date, transactions::amount))
                .order(transactions::date.desc())
                .load(db_conn)
                .with_context(|| format!("Unable to get income for {}", user))?;

            let mut monthly_totals = vec![];
            if !entries.is_empty() {
                let first_entry = entries.first().unwrap();
                let mut current_total = MonthlyTotal {
                    month: NaiveDate::from_ymd_opt(
                        first_entry.date.year(),
                        first_entry.date.month(),
                        1,
                    )
                    .unwrap(),
                    income: Decimal::ZERO,
                    expense: Decimal::ZERO,
                };

                for entry in entries.into_iter() {
                    if entry.date.month() != current_total.month.month()
                        || entry.date.year() != current_total.month.year()
                    {
                        monthly_totals.push(current_total);
                        current_total = MonthlyTotal::new(
                            NaiveDate::from_ymd_opt(entry.date.year(), entry.date.month(), 1)
                                .unwrap(),
                            Decimal::ZERO,
                            Decimal::ZERO,
                        )
                    }

                    if entry.amount > Decimal::ZERO {
                        current_total.income += entry.amount;
                    } else {
                        current_total.expense -= entry.amount;
                    }
                }
                monthly_totals.push(current_total);
            }

            Ok(monthly_totals)
        })
        .await
    }

    #[instrument(skip(self))]
    async fn get_all_categories(&self, user: &str) -> Result<Vec<String>, TransactionRepoError> {
        let user = user.to_owned();
        self.block(move |db_conn| {
            use crate::diesel_repo::schema::transactions::dsl::*;

            let categories = transactions
                .filter(user_id.eq(&user))
                .select(category)
                .distinct()
                .load::<String>(db_conn)
                .with_context(|| format!("Unable to get all categories for user {}", user))?;
            Ok(categories)
        })
        .await
    }

    #[instrument(skip(self))]
    async fn get_all_tags(&self, user: &str) -> Result<Vec<String>, TransactionRepoError> {
        let user = user.to_owned();
        self.block(move |db_conn| {
            use crate::diesel_repo::schema::transactions::dsl::*;

            let all_tags: Vec<String> = transactions
                .filter(user_id.eq(&user))
                .select(sql::<Text>("DISTINCT UNNEST(tags)"))
                .load(db_conn)
                .with_context(|| format!("Unable to get all tags for user {}", user))?;

            Ok(all_tags)
        })
        .await
    }

    #[instrument(skip(self))]
    async fn get_all_transactees(
        &self,
        user: &str,
        category: Option<String>,
    ) -> Result<Vec<String>, TransactionRepoError> {
        let user = user.to_owned();
        self.block(move |db_conn| {
            use diesel::dsl::count;

            let transactees: Vec<String> = if let Some(category) = category {
                let transactees: Vec<Option<String>> = transactions::table
                    .filter(transactions::user_id.eq(&user))
                    .filter(transactions::transactee.is_not_null())
                    .select(transactions::transactee)
                    .distinct()
                    .load::<Option<String>>(db_conn)
                    .with_context(|| format!("Unable to get all transactees for user {}", user))?;
                // remove null entry if there is one
                let mut transactees: Vec<String> = transactees.into_iter().flatten().collect();

                let mut transactee_counts: HashMap<String, i64> =
                    transactees.iter().cloned().map(|t| (t, 0)).collect();
                let category_transactee_counts: Vec<(Option<String>, i64)> = transactions::table
                    .filter(transactions::user_id.eq(&user))
                    .filter(transactions::transactee.eq(&category))
                    .filter(transactions::transactee.is_not_null())
                    .group_by(transactions::transactee)
                    .select((transactions::transactee, count(transactions::transactee)))
                    .load::<(Option<String>, i64)>(db_conn)
                    .with_context(|| {
                        format!(
                            "Unable to get transaction counts for transactee with category {}",
                            category
                        )
                    })?;
                for (transactee, count) in category_transactee_counts {
                    let Some(transactee) = transactee else {
                        continue;
                    };

                    let t_count = transactee_counts
                        .get_mut(&transactee)
                        .expect("Transactee should be present in all transactees");
                    *t_count += count;
                }

                transactees.sort_by(|a, b| transactee_counts.get(b).cmp(&transactee_counts.get(a)));
                transactees
            } else {
                let results = transactions::table
                    .filter(transactions::user_id.eq(&user))
                    .filter(transactions::transactee.is_not_null())
                    .group_by(transactions::transactee)
                    .order_by(count(transactions::transactee).desc())
                    .select(transactions::transactee)
                    .load::<Option<String>>(db_conn)
                    .with_context(|| format!("Unable to get all transactees for user {}", user))?;
                // remove null entry if there is one
                results.into_iter().flatten().collect()
            };
            Ok(transactees)
        })
        .await
    }

    #[instrument(skip(self))]
    async fn get_balance(&self, user: &str) -> Result<Decimal, TransactionRepoError> {
        let user = user.to_owned();
        self.block(move |db_conn| {
            use crate::diesel_repo::schema::transactions::dsl::*;

            let balance: Option<Decimal> = transactions
                .filter(user_id.eq(&user))
                .select(sum(amount))
                .first(db_conn)
                .with_context(|| format!("Unable to get balance for user {}", user))?;
            Ok(balance.unwrap_or(Decimal::ZERO))
        })
        .await
    }
}
