use diesel::prelude::*;
use rocket::http::Status;
use rocket_contrib::json::Json;

use crate::models::{NewTransaction, Transaction};
use crate::DBConnection;

#[get("/<transaction_id>")]
pub fn get_transaction(db_conn: DBConnection, transaction_id: i32) -> Option<Json<Transaction>> {
    use crate::schema::transactions::dsl::*;
    let transaction = transactions.find(transaction_id).first(&*db_conn).ok();
    transaction.map(|transaction| Json(transaction))
}

#[get("/")]
pub fn get_all_transactions(db_conn: DBConnection) -> Json<Vec<Transaction>> {
    use crate::schema::transactions::dsl::*;
    let all_transactions: Vec<Transaction> = transactions.load(&*db_conn).expect("No rows found");
    Json(all_transactions)
}

#[post("/", data = "<new_transaction>")]
pub fn create_new_transaction(
    db_conn: DBConnection,
    new_transaction: Json<NewTransaction>,
) -> Json<Transaction> {
    use crate::schema::transactions;
    let inserted_transaction: Transaction = diesel::insert_into(transactions::table)
        .values(new_transaction.into_inner())
        .get_result(&*db_conn)
        .expect("Unable to insert row");
    Json(inserted_transaction)
}

#[put("/<transaction_id>", data = "<updated_transaction>")]
pub fn update_transaction(
    db_conn: DBConnection,
    transaction_id: i32,
    updated_transaction: Json<NewTransaction>,
) -> Result<Json<Transaction>, Status> {
    use crate::schema::transactions::dsl::*;
    let result: QueryResult<Transaction> = diesel::update(transactions.find(transaction_id))
        .set(updated_transaction.into_inner())
        .get_result(&*db_conn);

    result
        .map(|transaction| Json(transaction))
        .map_err(|e| match e {
            diesel::NotFound => Status::NotFound,
            _ => Status::InternalServerError,
        })
}

#[delete("/<transaction_id>")]
pub fn delete_transaction(
    db_conn: DBConnection,
    transaction_id: i32,
) -> Result<Json<Transaction>, Status> {
    use crate::schema::transactions::dsl::*;
    let result: QueryResult<Transaction> =
        diesel::delete(transactions.find(transaction_id)).get_result(&*db_conn);

    result
        .map(|transaction| Json(transaction))
        .map_err(|e| match e {
            diesel::NotFound => Status::NotFound,
            _ => Status::InternalServerError,
        })
}
