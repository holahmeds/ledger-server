use actix_web::error::{BlockingError, ErrorInternalServerError, ErrorNotFound};
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web;

use crate::DbPool;
use crate::models;
use crate::models::NewTransaction;

#[get("/{transaction_id}")]
pub async fn get_transaction(
    pool: web::Data<DbPool>,
    web::Path(transaction_id): web::Path<i32>,
) -> impl Responder {
    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result = web::block(move || {
        models::get_transaction(&conn, transaction_id)
    })
        .await;

    result
        .map(|transaction| HttpResponse::Ok().json(transaction))
        .map_err(|e| match e {
            BlockingError::Error(x) => match x {
                diesel::NotFound => ErrorNotFound(x),
                _ => ErrorInternalServerError(x),
            },
            _ => ErrorInternalServerError(e),
        })
}

#[get("")]
pub async fn get_all_transactions(pool: web::Data<DbPool>) -> impl Responder {
    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result = web::block(move || models::get_all_transactions(&conn)).await;

    result
        .map(|transactions| HttpResponse::Ok().json(transactions))
        .map_err(|e| ErrorInternalServerError(e))
}

#[post("")]
pub async fn create_new_transaction(
    pool: web::Data<DbPool>,
    new_transaction: web::Json<NewTransaction>,
) -> impl Responder {
    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result =
        web::block(move || models::create_new_transaction(&conn, (*new_transaction).clone())).await;

    result
        .map(|transaction| HttpResponse::Ok().json(transaction))
        .map_err(|e| ErrorInternalServerError(e))
}

#[put("/{transaction_id}")]
pub async fn update_transaction(
    pool: web::Data<DbPool>,
    web::Path(transaction_id): web::Path<i32>,
    updated_transaction: web::Json<NewTransaction>,
) -> impl Responder {
    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result = web::block(move || {
        models::update_transaction(&conn, transaction_id, (*updated_transaction).clone())
    })
        .await;

    result
        .map(|updated_transaction| HttpResponse::Ok().json(updated_transaction))
        .map_err(|e| match e {
            BlockingError::Error(x) => match x {
                diesel::NotFound => ErrorNotFound(x),
                _ => ErrorInternalServerError(x),
            },
            _ => ErrorInternalServerError(e),
        })
}

#[delete("/{transaction_id}")]
pub async fn delete_transaction(
    pool: web::Data<DbPool>,
    web::Path(transaction_id): web::Path<i32>,
) -> impl Responder {
    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result = web::block(move || models::delete_transaction(&conn, transaction_id)).await;

    result
        .map(|deleted_transaction| HttpResponse::Ok().json(deleted_transaction))
        .map_err(|e| match e {
            BlockingError::Error(x) => match x {
                diesel::NotFound => ErrorNotFound(x),
                _ => ErrorInternalServerError(x),
            },
            _ => ErrorInternalServerError(e),
        })
}
