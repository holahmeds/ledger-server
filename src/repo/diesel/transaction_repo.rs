use super::schema::{transaction_tags, transactions};
use super::DbPool;
use crate::repo::transaction_repo::{
    NewTransaction, Transaction, TransactionRepo, TransactionRepoError,
};
use crate::user::UserId;
use actix_web::web;
use anyhow::Context;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::{Connection, PgConnection, QueryDsl, RunQueryDsl};
use r2d2::PooledConnection;
use rust_decimal::Decimal;

#[derive(Queryable, Identifiable)]
#[table_name = "transactions"]
struct TransactionEntry {
    id: i32,
    category: String,
    transactee: Option<String>,
    note: Option<String>,
    date: NaiveDate,
    amount: Decimal,
    user_id: UserId,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "transactions"]
struct NewTransactionEntry {
    category: String,
    transactee: Option<String>,
    note: Option<String>,
    date: NaiveDate,
    amount: Decimal,
    user_id: UserId,
}

impl Transaction {
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

impl NewTransaction {
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

#[derive(Associations, Identifiable, Queryable, Insertable)]
#[primary_key(transaction_id, tag)]
#[belongs_to(TransactionEntry, foreign_key = "transaction_id")]
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
                PooledConnection<ConnectionManager<PgConnection>>,
            ) -> Result<R, TransactionRepoError>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let pool = self.db_pool.clone();
        web::block(move || {
            let db_conn = pool.get().context("Unable to get connection from pool")?;
            f(db_conn)
        })
        .await
        .context("Blocking error")?
    }
}

#[async_trait]
impl TransactionRepo for DieselTransactionRepo {
    async fn get_transaction(
        &self,
        user: UserId,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError> {
        self.block(move |db_conn| {
            use crate::repo::diesel::schema::transactions::dsl::*;
            use diesel::QueryDsl;

            let transaction_entry = transactions
                .find(transaction_id)
                .filter(user_id.eq(user))
                .get_result(&db_conn)
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => {
                        TransactionRepoError::TransactionNotFound(transaction_id)
                    }
                    _ => TransactionRepoError::Other(anyhow::Error::new(e).context(format!(
                        "Unable to get transaction {} from database",
                        transaction_id
                    ))),
                })?;
            let transaction_tags = get_tags(&db_conn, transaction_id).with_context(|| {
                format!("Unable to get tags for transaction {}", transaction_id)
            })?;

            Ok(Transaction::from_entry_and_tags(
                transaction_entry,
                transaction_tags,
            ))
        })
        .await
    }

    async fn get_all_transactions(
        &self,
        user: UserId,
        from: Option<NaiveDate>,
        until: Option<NaiveDate>,
        category: Option<String>,
        transactee: Option<String>,
    ) -> Result<Vec<Transaction>, TransactionRepoError> {
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

            let transactions_entries = query
                .order((transactions::date.desc(), transactions::id.desc()))
                .load(&db_conn)
                .context("Unable to retrieve transactions")?;
            let transaction_tags = TransactionTag::belonging_to(&transactions_entries)
                .load::<TransactionTag>(&db_conn)
                .context("Unable to retrieve tags for transactions")?
                .grouped_by(&transactions_entries);

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

    async fn create_new_transaction(
        &self,
        user: UserId,
        new_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError> {
        let (new_transaction_entry, tags) = new_transaction.split_tags(user);

        self.block(move |db_conn| {
            let transaction_entry = db_conn
                .transaction::<_, diesel::result::Error, _>(|| {
                    let transaction_entry: TransactionEntry =
                        diesel::insert_into(transactions::table)
                            .values(new_transaction_entry)
                            .get_result(&db_conn)?;

                    add_tags(&db_conn, transaction_entry.id, tags.clone())?;
                    Ok(transaction_entry)
                })
                .context("Unable to insert transaction")?;
            Ok(Transaction::from_entry_and_tags(transaction_entry, tags))
        })
        .await
    }

    async fn update_transaction(
        &self,
        user: UserId,
        transaction_id: i32,
        updated_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError> {
        let (new_transaction_entry, updated_tags) = updated_transaction.split_tags(user.clone());

        self.block(move |db_conn| {
            let transaction_entry = db_conn
                .transaction(|| {
                    let transaction_entry = diesel::update(
                        transactions::table
                            .find(transaction_id)
                            .filter(transactions::user_id.eq(user)),
                    )
                    .set(new_transaction_entry)
                    .get_result(&db_conn)?;

                    let existing_tags: Vec<String> = get_tags(&db_conn, transaction_id)?;

                    let new_tags: Vec<String> = updated_tags
                        .clone()
                        .into_iter()
                        .filter(|t| !existing_tags.contains(t))
                        .collect();
                    add_tags(&db_conn, transaction_id, new_tags)?;

                    let removed_tags: Vec<&String> = existing_tags
                        .iter()
                        .filter(|t| !updated_tags.contains(t))
                        .collect();
                    diesel::delete(
                        transaction_tags::table
                            .filter(transaction_tags::transaction_id.eq(transaction_id))
                            .filter(transaction_tags::tag.eq_any(removed_tags)),
                    )
                    .execute(&db_conn)?;

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

    async fn delete_transaction(
        &self,
        user: UserId,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError> {
        self.block(move |db_conn| {
            let tag_list = get_tags(&db_conn, transaction_id).with_context(|| {
                format!("Unable to get tags for transaction {}", transaction_id)
            })?;

            let transaction_entry = diesel::delete(
                transactions::table
                    .find(transaction_id)
                    .filter(transactions::user_id.eq(user)),
            )
            .get_result(&db_conn)
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

    async fn get_all_categories(&self, user: UserId) -> Result<Vec<String>, TransactionRepoError> {
        self.block(move |db_conn| {
            use crate::repo::diesel::schema::transactions::dsl::*;

            let categories = transactions
                .filter(user_id.eq(&user))
                .select(category)
                .distinct()
                .load::<String>(&db_conn)
                .with_context(|| format!("Unable to get all categories for user {}", user))?;
            Ok(categories)
        })
        .await
    }

    async fn get_all_tags(&self, user: UserId) -> Result<Vec<String>, TransactionRepoError> {
        self.block(move |db_conn| {
            use crate::repo::diesel::schema::transaction_tags::dsl::*;

            let tags = transaction_tags
                .left_join(transactions::table)
                .filter(transactions::user_id.eq(&user))
                .select(tag)
                .distinct()
                .load::<String>(&db_conn)
                .with_context(|| format!("Unable to get all tags for user {}", user))?;
            Ok(tags)
        })
        .await
    }

    async fn get_all_transactees(&self, user: UserId) -> Result<Vec<String>, TransactionRepoError> {
        self.block(move |db_conn| {
            use crate::repo::diesel::schema::transactions::dsl::*;

            let results = transactions
                .filter(user_id.eq(&user))
                .select(transactee)
                .distinct()
                .load::<Option<String>>(&db_conn)
                .with_context(|| format!("Unable to get all transactees for user {}", user))?;

            // remove null entry if there is one
            let transactees = results.into_iter().filter_map(|i| i).collect();
            Ok(transactees)
        })
        .await
    }
}

fn get_tags(
    db_conn: &PgConnection,
    transaction_id: i32,
) -> Result<Vec<String>, diesel::result::Error> {
    let tags = transaction_tags::table
        .filter(transaction_tags::transaction_id.eq(transaction_id))
        .select(transaction_tags::tag)
        .load::<String>(db_conn)?;
    Ok(tags)
}

fn add_tags(
    db_conn: &PgConnection,
    transaction_id: i32,
    tags: Vec<String>,
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