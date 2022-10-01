use anyhow::Context;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::Insertable;
use diesel::Queryable;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::schema::transaction_tags;
use crate::schema::transactions;
use crate::user::UserId;
use crate::DbPool;

use super::{NewTransaction, Transaction};

#[derive(Error, Debug)]
pub enum TransactionRepoError {
    #[error("Transaction with id {0} not found")]
    TransactionNotFound(i32),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Queryable, Identifiable)]
#[table_name = "transactions"]
pub struct TransactionEntry {
    pub id: i32,
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub user_id: UserId,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "transactions"]
pub struct NewTransactionEntry {
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub user_id: UserId,
}

#[derive(Associations, Identifiable, Queryable, Insertable)]
#[primary_key(transaction_id, tag)]
#[belongs_to(TransactionEntry, foreign_key = "transaction_id")]
struct TransactionTag {
    pub transaction_id: i32,
    pub tag: String,
}

pub fn get_transaction(
    pool: &DbPool,
    user: UserId,
    transaction_id: i32,
) -> Result<Transaction, TransactionRepoError> {
    use crate::schema::transactions::dsl::*;

    let db_conn = pool.get().context("Unable to get connection from pool")?;

    let transaction_entry: TransactionEntry = transactions
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
    let transaction_tags = get_tags(&db_conn, transaction_id)
        .with_context(|| format!("Unable to get tags for transaction {}", transaction_id))?;

    Ok(Transaction::from_entry_and_tags(
        transaction_entry,
        transaction_tags,
    ))
}

pub fn get_transactions(
    pool: &DbPool,
    user: UserId,
    from: Option<NaiveDate>,
    until: Option<NaiveDate>,
    category: Option<&str>,
    transactee: Option<&str>,
) -> Result<Vec<Transaction>, TransactionRepoError> {
    let db_conn = pool.get().context("Unable to get connection from pool")?;

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
}

pub fn create_new_transaction(
    pool: &DbPool,
    user: UserId,
    new_transaction: NewTransaction,
) -> Result<Transaction, TransactionRepoError> {
    let db_conn = pool.get().context("Unable to get connection from pool")?;

    let (new_transaction_entry, tags) = new_transaction.split_tags(user);

    let transaction_entry = db_conn
        .transaction::<_, diesel::result::Error, _>(|| {
            let transaction_entry: TransactionEntry = diesel::insert_into(transactions::table)
                .values(new_transaction_entry)
                .get_result(&db_conn)?;

            add_tags(&db_conn, transaction_entry.id, tags.clone())?;
            Ok(transaction_entry)
        })
        .context("Unable to insert transaction")?;

    Ok(Transaction::from_entry_and_tags(transaction_entry, tags))
}

pub fn update_transaction(
    pool: &DbPool,
    user: UserId,
    transaction_id: i32,
    updated_transaction: NewTransaction,
) -> Result<Transaction, TransactionRepoError> {
    let db_conn = pool.get().context("Unable to get connection from pool")?;

    let (new_transaction_entry, updated_tags) = updated_transaction.split_tags(user.clone());

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
}

pub fn delete_transaction(
    pool: &DbPool,
    user: UserId,
    transaction_id: i32,
) -> Result<Transaction, TransactionRepoError> {
    let db_conn = pool.get().context("Unable to get connection from pool")?;

    let tag_list = get_tags(&db_conn, transaction_id)
        .with_context(|| format!("Unable to get tags for transaction {}", transaction_id))?;

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
}

pub fn get_all_categories(
    pool: &DbPool,
    user: UserId,
) -> Result<Vec<String>, TransactionRepoError> {
    use crate::schema::transactions::dsl::*;

    let db_conn = pool.get().context("Unable to get connection from pool")?;

    let categories = transactions
        .filter(user_id.eq(&user))
        .select(category)
        .distinct()
        .load::<String>(&db_conn)
        .with_context(|| format!("Unable to get all categories for user {}", user))?;
    Ok(categories)
}

pub fn get_all_tags(pool: &DbPool, user: UserId) -> Result<Vec<String>, TransactionRepoError> {
    use crate::schema::transaction_tags::dsl::*;

    let db_conn = pool.get().context("Unable to get connection from pool")?;

    let tags = transaction_tags
        .left_join(transactions::table)
        .filter(transactions::user_id.eq(&user))
        .select(tag)
        .distinct()
        .load::<String>(&db_conn)
        .with_context(|| format!("Unable to get all tags for user {}", user))?;
    Ok(tags)
}

pub fn get_all_transactees(
    pool: &DbPool,
    user: UserId,
) -> Result<Vec<String>, TransactionRepoError> {
    use crate::schema::transactions::dsl::*;

    let db_conn = pool.get().context("Unable to get connection from pool")?;

    let results = transactions
        .filter(user_id.eq(&user))
        .select(transactee)
        .distinct()
        .load::<Option<String>>(&db_conn)
        .with_context(|| format!("Unable to get all transactees for user {}", user))?;

    // remove null entry if there is one
    let transactees = results.into_iter().filter_map(|i| i).collect();
    Ok(transactees)
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
        .clone()
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
