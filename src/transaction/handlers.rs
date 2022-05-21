use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::DbPool;

use super::models;
use super::NewTransaction;

#[get("/{transaction_id}")]
pub async fn get_transaction(pool: web::Data<DbPool>, params: web::Path<i32>) -> impl Responder {
    let transaction_id = params.into_inner();

    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result = web::block(move || models::get_transaction(&conn, transaction_id)).await?;

    result
        .map(|transaction| HttpResponse::Ok().json(transaction))
        .map_err(|e| match e {
            diesel::NotFound => ErrorNotFound(e),
            _ => ErrorInternalServerError(e),
        })
}

#[get("")]
pub async fn get_all_transactions(pool: web::Data<DbPool>) -> impl Responder {
    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result = web::block(move || models::get_all_transactions(&conn)).await?;

    result
        .map(|transactions| HttpResponse::Ok().json(transactions))
        .map_err(ErrorInternalServerError)
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
        web::block(move || models::create_new_transaction(&conn, (*new_transaction).clone()))
            .await?;

    result
        .map(|transaction| HttpResponse::Ok().json(transaction))
        .map_err(ErrorInternalServerError)
}

#[put("/{transaction_id}")]
pub async fn update_transaction(
    pool: web::Data<DbPool>,
    params: web::Path<i32>,
    updated_transaction: web::Json<NewTransaction>,
) -> impl Responder {
    let transaction_id = params.into_inner();

    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result = web::block(move || {
        models::update_transaction(&conn, transaction_id, (*updated_transaction).clone())
    })
    .await?;

    result
        .map(|updated_transaction| HttpResponse::Ok().json(updated_transaction))
        .map_err(|e| match e {
            diesel::NotFound => ErrorNotFound(e),
            _ => ErrorInternalServerError(e),
        })
}

#[delete("/{transaction_id}")]
pub async fn delete_transaction(pool: web::Data<DbPool>, params: web::Path<i32>) -> impl Responder {
    let transaction_id = params.into_inner();

    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result = web::block(move || models::delete_transaction(&conn, transaction_id)).await?;

    result
        .map(|deleted_transaction| HttpResponse::Ok().json(deleted_transaction))
        .map_err(|e| match e {
            diesel::NotFound => ErrorNotFound(e),
            _ => ErrorInternalServerError(e),
        })
}

#[get("/categories")]
pub async fn get_all_categories(pool: web::Data<DbPool>) -> impl Responder {
    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result = web::block(move || models::get_all_categories(&conn)).await?;

    result
        .map(|categories| HttpResponse::Ok().json(categories))
        .map_err(ErrorInternalServerError)
}

#[get("/tags")]
pub async fn get_all_tags(pool: web::Data<DbPool>) -> impl Responder {
    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result = web::block(move || models::get_all_tags(&conn)).await?;

    result
        .map(|tags| HttpResponse::Ok().json(tags))
        .map_err(ErrorInternalServerError)
}

#[get("/transactees")]
pub async fn get_all_transactees(pool: web::Data<DbPool>) -> impl Responder {
    let conn = pool
        .get()
        .expect("Unable to get database connection from pool");

    let result = web::block(move || models::get_all_transactees(&conn)).await?;

    result
        .map(|transactees| HttpResponse::Ok().json(transactees))
        .map_err(ErrorInternalServerError)
}
