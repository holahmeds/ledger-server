use super::schema::{transaction_tags, transactions};
use super::DbPool;
use crate::transaction_repo::{
    MonthlyTotal, NewTransaction, PageOptions, Transaction, TransactionRepo, TransactionRepoError,
};
use actix_web::web;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use diesel::dsl::sum;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::{Connection, PgConnection, QueryDsl, RunQueryDsl};
use r2d2::PooledConnection;
use rust_decimal::Decimal;
use std::collections::HashSet;
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
}

impl Transaction {
    fn from_entry_and_tags(
        transaction_entry: TransactionEntry,
        tags: HashSet<String>,
    ) -> Transaction {
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

impl NewTransaction {
    fn split_tags(self, user_id: String) -> (NewTransactionEntry, HashSet<String>) {
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

    #[instrument(skip(db_conn))]
    fn delete_tags(
        db_conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
        transaction_id: i32,
        removed_tags: Vec<&String>,
    ) -> Result<(), diesel::result::Error> {
        diesel::delete(
            transaction_tags::table
                .filter(transaction_tags::transaction_id.eq(transaction_id))
                .filter(transaction_tags::tag.eq_any(removed_tags)),
        )
        .execute(db_conn)?;
        Ok(())
    }

    fn get_tags_multi(
        db_conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
        transactions_entries: &Vec<TransactionEntry>,
    ) -> Result<Vec<Vec<TransactionTag>>, TransactionRepoError> {
        let tags = TransactionTag::belonging_to(transactions_entries)
            .load::<TransactionTag>(db_conn)
            .context("Unable to retrieve tags for transactions")?
            .grouped_by(&transactions_entries);
        Ok(tags)
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

            let transaction_entry = transactions
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
            let transaction_tags = get_tags(db_conn, transaction_id).with_context(|| {
                format!("Unable to get tags for transaction {}", transaction_id)
            })?;

            Ok(Transaction::from_entry_and_tags(
                transaction_entry,
                transaction_tags,
            ))
        })
        .await
    }

    #[instrument(skip(self))]
    async fn get_all_transactions(
        &self,
        user: &str,
        from: Option<NaiveDate>,
        until: Option<NaiveDate>,
        category: Option<String>,
        transactee: Option<String>,
        page_options: Option<PageOptions>,
    ) -> Result<Vec<Transaction>, TransactionRepoError> {
        let user = user.to_owned();
        self.block(move |db_conn| {
            let mut query = transactions::table
                .filter(transactions::user_id.eq(user))
                .into_boxed();
            if let Some(from) = from {
                query = query.filter(transactions::date.ge(from))
            }
            if let Some(until) = until {
                query = query.filter(transactions::date.le(until))
            }
            if let Some(category) = category {
                query = query.filter(transactions::category.eq(category))
            }
            if let Some(transactee) = transactee {
                query = query.filter(transactions::transactee.eq(transactee))
            }
            if let Some(po) = page_options {
                query = query.offset(po.offset).limit(po.limit)
            }

            let transactions_entries = query
                .order((transactions::date.desc(), transactions::id.desc()))
                .load(db_conn)
                .context("Unable to retrieve transactions")?;
            let transaction_tags = Self::get_tags_multi(db_conn, &transactions_entries)?;

            let transactions_list = transactions_entries
                .into_iter()
                .zip(transaction_tags)
                .map(|(transaction_entry, transaction_tag_list)| {
                    let tags = transaction_tag_list.into_iter().map(|tt| tt.tag).collect();
                    Transaction::from_entry_and_tags(transaction_entry, tags)
                })
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
        let (new_transaction_entry, tags) = new_transaction.split_tags(user.to_owned());

        self.block(move |db_conn| {
            let transaction_entry = db_conn
                .transaction::<_, diesel::result::Error, _>(|db_conn| {
                    let transaction_entry: TransactionEntry =
                        diesel::insert_into(transactions::table)
                            .values(new_transaction_entry)
                            .get_result(db_conn)?;

                    add_tags(db_conn, transaction_entry.id, tags.clone())?;
                    Ok(transaction_entry)
                })
                .context("Unable to insert transaction")?;
            Ok(Transaction::from_entry_and_tags(transaction_entry, tags))
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
        let (new_transaction_entry, updated_tags) = updated_transaction.split_tags(user.to_owned());

        self.block(move |db_conn| {
            let transaction_entry = db_conn
                .transaction(|db_conn| {
                    let transaction_entry = diesel::update(
                        transactions::table
                            .find(transaction_id)
                            .filter(transactions::user_id.eq(&new_transaction_entry.user_id)),
                    )
                    .set(&new_transaction_entry)
                    .get_result(db_conn)?;

                    let existing_tags: HashSet<String> = get_tags(db_conn, transaction_id)?;

                    let new_tags: HashSet<String> = updated_tags
                        .clone()
                        .into_iter()
                        .filter(|t| !existing_tags.contains(t))
                        .collect();
                    add_tags(db_conn, transaction_id, new_tags)?;

                    let removed_tags: Vec<&String> =
                        existing_tags.difference(&updated_tags).collect();
                    Self::delete_tags(db_conn, transaction_id, removed_tags)?;

                    Ok(transaction_entry)
                })
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => {
                        TransactionRepoError::TransactionNotFound(transaction_id)
                    }
                    _ => TransactionRepoError::Other(
                        anyhow::Error::new(e)
                            .context(format!("Unable to update transaction {}", transaction_id)),
                    ),
                })?;

            Ok(Transaction::from_entry_and_tags(
                transaction_entry,
                updated_tags,
            ))
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
            let tag_list = get_tags(db_conn, transaction_id).with_context(|| {
                format!("Unable to get tags for transaction {}", transaction_id)
            })?;

            let transaction_entry = diesel::delete(
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

            Ok(Transaction::from_entry_and_tags(
                transaction_entry,
                tag_list,
            ))
        })
        .await
    }

    #[instrument(skip(self))]
    async fn get_monthly_totals(
        &self,
        user: &str,
    ) -> Result<Vec<MonthlyTotal>, TransactionRepoError> {
        let user = user.to_owned();
        self.block(move |db_conn| {
            #[derive(Queryable)]
            struct Entry {
                date: NaiveDate,
                amount: Decimal,
            }

            let entries: Vec<Entry> = transactions::table
                .filter(transactions::user_id.eq(&user))
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
            use crate::diesel_repo::schema::transaction_tags::dsl::*;

            let tags = transaction_tags
                .left_join(transactions::table)
                .filter(transactions::user_id.eq(&user))
                .select(tag)
                .distinct()
                .load::<String>(db_conn)
                .with_context(|| format!("Unable to get all tags for user {}", user))?;
            Ok(tags)
        })
        .await
    }

    #[instrument(skip(self))]
    async fn get_all_transactees(&self, user: &str) -> Result<Vec<String>, TransactionRepoError> {
        let user = user.to_owned();
        self.block(move |db_conn| {
            use crate::diesel_repo::schema::transactions::dsl::*;
            use diesel::dsl::count;

            let results = transactions
                .filter(user_id.eq(&user))
                .filter(transactee.is_not_null())
                .group_by(transactee)
                .order_by(count(transactee).desc())
                .select(transactee)
                .load::<Option<String>>(db_conn)
                .with_context(|| format!("Unable to get all transactees for user {}", user))?;

            // remove null entry if there is one
            let transactees = results.into_iter().flatten().collect();
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

#[instrument(skip(db_conn))]
fn get_tags(
    db_conn: &mut PgConnection,
    transaction_id: i32,
) -> Result<HashSet<String>, diesel::result::Error> {
    let tags = transaction_tags::table
        .filter(transaction_tags::transaction_id.eq(transaction_id))
        .select(transaction_tags::tag)
        .load::<String>(db_conn)?;
    Ok(HashSet::from_iter(tags))
}

#[instrument(skip(db_conn))]
fn add_tags(
    db_conn: &mut PgConnection,
    transaction_id: i32,
    tags: HashSet<String>,
) -> Result<(), diesel::result::Error> {
    let transaction_tag_list: Vec<TransactionTag> = tags
        .into_iter()
        .map(|tag| TransactionTag {
            transaction_id,
            tag,
        })
        .collect();
    diesel::insert_into(transaction_tags::table)
        .values(transaction_tag_list)
        .execute(db_conn)?;
    Ok(())
}
