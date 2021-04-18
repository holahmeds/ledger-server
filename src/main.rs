#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;

use rocket_contrib::json::Json;
use serde::Serialize;

static TRANSACTIONS: [Transaction; 2] = [
    Transaction::new(1, "Misc", "Bob", None),
    Transaction::new(2, "Misc", "Alice", None)
];

#[derive(Serialize,Clone)]
struct Transaction<'a> {
    id: i32,
    category: &'a str,
    transactee: &'a str,
    note: Option<&'a str>,
}

impl Transaction<'_> {
    const fn new<'a>(id: i32, category: &'a str, transactee: &'a str, note: Option<&'a str>) -> Transaction<'a> {
        Transaction {
            id,
            category,
            transactee,
            note,
        }
    }
}

#[get("/transactions/<id>")]
fn get_transaction(id: i32) -> Option<Json<Transaction<'static>>> {
    let transaction = TRANSACTIONS.iter().find(|transaction| transaction.id == id).cloned();
    transaction.map(|transaction| Json(transaction))
}

#[get("/transactions")]
fn get_all_transactions() -> Json<&'static [Transaction<'static>]> {
    Json(&TRANSACTIONS)
}

fn main() {
    rocket::ignite().mount("/", routes![get_transaction, get_all_transactions]).launch();
}
