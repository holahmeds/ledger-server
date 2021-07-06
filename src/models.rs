use chrono::NaiveDate;
use diesel::Insertable;
use diesel::prelude::*;
use diesel::Queryable;
use diesel::result::Error;
use serde::Deserialize;
use serde::Serialize;

use super::schema::transactions;

#[derive(Serialize, Deserialize, Clone, Queryable, PartialEq, Debug)]
pub struct Transaction {
    pub id: i32,
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub transaction_date: NaiveDate,
}

#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[table_name = "transactions"]
pub struct NewTransaction {
    pub category: String,
    pub transactee: Option<String>,
    pub note: Option<String>,
    pub transaction_date: NaiveDate,
}

impl Transaction {
    pub const fn new(
        id: i32,
        category: String,
        transactee: Option<String>,
        note: Option<String>,
        transaction_date: NaiveDate,
    ) -> Transaction {
        Transaction {
            id,
            category,
            transactee,
            note,
            transaction_date,
        }
    }
}

impl NewTransaction {
    pub const fn new(
        category: String,
        transactee: Option<String>,
        note: Option<String>,
        transaction_date: NaiveDate,
    ) -> NewTransaction {
        NewTransaction {
            category,
            transactee,
            note,
            transaction_date,
        }
    }
}

pub fn get_transaction(db_conn: &PgConnection, transaction_id: i32) -> Result<Transaction, Error> {
    use crate::schema::transactions::dsl::*;
    transactions.find(transaction_id).first(db_conn)
}

pub fn get_all_transactions(db_conn: &PgConnection) -> Result<Vec<Transaction>, Error> {
    use crate::schema::transactions::dsl::*;
    transactions.load(db_conn)
}

pub fn create_new_transaction(
    db_conn: &PgConnection,
    new_transaction: NewTransaction,
) -> Result<Transaction, Error> {
    diesel::insert_into(transactions::table)
        .values(new_transaction)
        .get_result(db_conn)
}

pub fn update_transaction(
    db_conn: &PgConnection,
    transaction_id: i32,
    updated_transaction: NewTransaction,
) -> Result<Transaction, Error> {
    use crate::schema::transactions::dsl::*;

    diesel::update(transactions.find(transaction_id))
        .set(updated_transaction)
        .get_result(db_conn)
}

pub fn delete_transaction(
    db_conn: &PgConnection,
    transaction_id: i32,
) -> Result<Transaction, Error> {
    use crate::schema::transactions::dsl::*;

    diesel::delete(transactions.find(transaction_id)).get_result(db_conn)
}
