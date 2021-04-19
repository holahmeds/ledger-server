use diesel::Insertable;
use diesel::Queryable;
use serde::Deserialize;
use serde::Serialize;

use super::schema::transactions;

#[derive(Serialize, Clone, Queryable)]
pub struct Transaction {
    pub id: i32,
    pub category: String,
    pub transactee: String,
    pub note: Option<String>,
}

#[derive(Deserialize, Insertable, AsChangeset)]
#[table_name = "transactions"]
pub struct NewTransaction<'a> {
    pub category: &'a str,
    pub transactee: &'a str,
    pub note: Option<&'a str>,
}

impl Transaction {
    pub const fn new(
        id: i32,
        category: String,
        transactee: String,
        note: Option<String>,
    ) -> Transaction {
        Transaction {
            id,
            category,
            transactee,
            note,
        }
    }
}
