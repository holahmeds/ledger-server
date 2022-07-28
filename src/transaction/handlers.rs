use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::error::HandlerError;
use crate::DbPool;

use super::models;
use super::NewTransaction;

#[get("/{transaction_id}")]
pub async fn get_transaction(
    pool: web::Data<DbPool>,
    transaction_id: web::Path<i32>,
) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    let transaction =
        web::block(move || models::get_transaction(&conn, transaction_id.into_inner())).await??;
    Ok(HttpResponse::Ok().json(transaction))
}

#[get("")]
pub async fn get_all_transactions(pool: web::Data<DbPool>) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    let transaction = web::block(move || models::get_all_transactions(&conn)).await??;
    Ok(HttpResponse::Ok().json(transaction))
}

#[post("")]
pub async fn create_new_transaction(
    pool: web::Data<DbPool>,
    new_transaction: web::Json<NewTransaction>,
) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    let transaction =
        web::block(move || models::create_new_transaction(&conn, new_transaction.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(transaction))
}

#[put("/{transaction_id}")]
pub async fn update_transaction(
    pool: web::Data<DbPool>,
    transaction_id: web::Path<i32>,
    updated_transaction: web::Json<NewTransaction>,
) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    let transaction = web::block(move || {
        models::update_transaction(
            &conn,
            transaction_id.into_inner(),
            updated_transaction.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(transaction))
}

#[delete("/{transaction_id}")]
pub async fn delete_transaction(
    pool: web::Data<DbPool>,
    transaction_id: web::Path<i32>,
) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    let transaction =
        web::block(move || models::delete_transaction(&conn, transaction_id.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(transaction))
}

#[get("/categories")]
pub async fn get_all_categories(pool: web::Data<DbPool>) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    let categories = web::block(move || models::get_all_categories(&conn)).await??;
    Ok(HttpResponse::Ok().json(categories))
}

#[get("/tags")]
pub async fn get_all_tags(pool: web::Data<DbPool>) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    let tags = web::block(move || models::get_all_tags(&conn)).await??;
    Ok(HttpResponse::Ok().json(tags))
}

#[get("/transactees")]
pub async fn get_all_transactees(pool: web::Data<DbPool>) -> Result<impl Responder, HandlerError> {
    let conn = pool.get()?;

    let transactees = web::block(move || models::get_all_transactees(&conn)).await??;
    Ok(HttpResponse::Ok().json(transactees))
}
