use chrono::NaiveDate;
use diesel::Insertable;
use diesel::prelude::*;
use diesel::Queryable;
use diesel::result::Error;
use rust_decimal::Decimal;

use crate::schema::transaction_tags;
use crate::schema::transactions;

use super::{NewTransaction, Transaction};

#[derive(Queryable, Identifiable)]
#[table_name = "transactions"]
pub struct TransactionEntry {
    pub id: i32,
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub date: NaiveDate,
    pub amount: Decimal,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "transactions"]
pub struct NewTransactionEntry {
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub date: NaiveDate,
    pub amount: Decimal,
}

#[derive(Associations, Identifiable, Queryable, Insertable)]
#[primary_key(transaction_id, tag)]
#[belongs_to(TransactionEntry, foreign_key = "transaction_id")]
struct TransactionTag {
    pub transaction_id: i32,
    pub tag: String,
}

pub fn get_transaction(db_conn: &PgConnection, transaction_id: i32) -> Result<Transaction, Error> {
    use crate::schema::transaction_tags::columns::tag;
    use crate::schema::transactions::dsl::*;

    let transaction_entry: TransactionEntry = transactions.find(transaction_id).first(db_conn)?;
    let transaction_tags = TransactionTag::belonging_to(&transaction_entry)
        .select(tag)
        .load::<String>(db_conn)?;

    Ok(Transaction::from_entry_and_tags(
        transaction_entry,
        transaction_tags,
    ))
}

pub fn get_all_transactions(db_conn: &PgConnection) -> Result<Vec<Transaction>, Error> {
    let transactions_entries = transactions::table.load(db_conn)?;
    let transaction_tags = TransactionTag::belonging_to(&transactions_entries)
        .load::<TransactionTag>(db_conn)?
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
    db_conn: &PgConnection,
    new_transaction: NewTransaction,
) -> Result<Transaction, Error> {
    let (new_transaction_entry, tags) = new_transaction.split_tags();

    let transaction_entry: TransactionEntry = diesel::insert_into(transactions::table)
        .values(new_transaction_entry)
        .get_result(db_conn)?;

    let transaction_tag_list: Vec<TransactionTag> = tags
        .clone()
        .into_iter()
        .map(|t| TransactionTag {
            transaction_id: transaction_entry.id,
            tag: t,
        })
        .collect();
    diesel::insert_into(transaction_tags::table)
        .values(transaction_tag_list)
        .execute(db_conn)?;

    Ok(Transaction::from_entry_and_tags(transaction_entry, tags))
}

pub fn update_transaction(
    db_conn: &PgConnection,
    transaction_id: i32,
    updated_transaction: NewTransaction,
) -> Result<Transaction, Error> {
    let (new_transaction_entry, updated_tags) = updated_transaction.split_tags();

    let transaction_entry = diesel::update(transactions::table.find(transaction_id))
        .set(new_transaction_entry)
        .get_result(db_conn)?;

    let existing_tags: Vec<String> = transaction_tags::table
        .filter(transaction_tags::transaction_id.eq(transaction_id))
        .select(transaction_tags::tag)
        .load(db_conn)?;

    let new_tags: Vec<TransactionTag> = updated_tags
        .clone()
        .into_iter()
        .filter(|t| !existing_tags.contains(t))
        .map(|t| TransactionTag {
            transaction_id,
            tag: t,
        })
        .collect();
    diesel::insert_into(transaction_tags::table)
        .values(new_tags)
        .execute(db_conn)?;

    let removed_tags: Vec<&String> = existing_tags
        .iter()
        .filter(|t| !updated_tags.contains(t))
        .collect();
    diesel::delete(
        transaction_tags::table
            .filter(transaction_tags::transaction_id.eq(transaction_id))
            .filter(transaction_tags::tag.eq_any(removed_tags)),
    )
        .execute(db_conn)?;

    Ok(Transaction::from_entry_and_tags(
        transaction_entry,
        updated_tags,
    ))
}

pub fn delete_transaction(
    db_conn: &PgConnection,
    transaction_id: i32,
) -> Result<Transaction, Error> {
    let tag_list = transaction_tags::table
        .filter(transaction_tags::transaction_id.eq(transaction_id))
        .select(transaction_tags::tag)
        .load::<String>(db_conn)?;

    let transaction_entry =
        diesel::delete(transactions::table.find(transaction_id)).get_result(db_conn)?;

    Ok(Transaction::from_entry_and_tags(
        transaction_entry,
        tag_list,
    ))
}
