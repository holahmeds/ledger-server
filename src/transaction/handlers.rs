use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use anyhow::Context;
use chrono::NaiveDate;
use serde::Deserialize;

use crate::error::HandlerError;
use crate::user::UserId;
use crate::DbPool;

use super::models;
use super::NewTransaction;

#[derive(Deserialize)]
pub struct Filter {
    from: Option<NaiveDate>,
    until: Option<NaiveDate>,
    category: Option<String>,
    transactee: Option<String>,
}

#[get("/{transaction_id}")]
pub async fn get_transaction(
    pool: web::Data<DbPool>,
    user_id: web::ReqData<UserId>,
    transaction_id: web::Path<i32>,
) -> Result<impl Responder, HandlerError> {
    let transaction = web::block(move || {
        models::get_transaction(&pool, user_id.into_inner(), transaction_id.into_inner())
    })
    .await
    .context("Blocking error")??;
    Ok(HttpResponse::Ok().json(transaction))
}

#[get("")]
pub async fn get_transactions(
    pool: web::Data<DbPool>,
    user_id: web::ReqData<UserId>,
    filter: web::Query<Filter>,
) -> Result<impl Responder, HandlerError> {
    let transaction = web::block(move || {
        models::get_transactions(
            &pool,
            user_id.into_inner(),
            filter.from,
            filter.until,
            filter.category.as_deref(),
            filter.transactee.as_deref(),
        )
    })
    .await
    .context("Blocking error")??;
    Ok(HttpResponse::Ok().json(transaction))
}

#[post("")]
pub async fn create_new_transaction(
    pool: web::Data<DbPool>,
    user_id: web::ReqData<UserId>,
    new_transaction: web::Json<NewTransaction>,
) -> Result<impl Responder, HandlerError> {
    let transaction = web::block(move || {
        models::create_new_transaction(&pool, user_id.into_inner(), new_transaction.into_inner())
    })
    .await
    .context("Blocking error")??;
    Ok(HttpResponse::Ok().json(transaction))
}

#[put("/{transaction_id}")]
pub async fn update_transaction(
    pool: web::Data<DbPool>,
    user_id: web::ReqData<UserId>,
    transaction_id: web::Path<i32>,
    updated_transaction: web::Json<NewTransaction>,
) -> Result<impl Responder, HandlerError> {
    let transaction = web::block(move || {
        models::update_transaction(
            &pool,
            user_id.into_inner(),
            transaction_id.into_inner(),
            updated_transaction.into_inner(),
        )
    })
    .await
    .context("Blocking error")??;
    Ok(HttpResponse::Ok().json(transaction))
}

#[delete("/{transaction_id}")]
pub async fn delete_transaction(
    pool: web::Data<DbPool>,
    user_id: web::ReqData<UserId>,
    transaction_id: web::Path<i32>,
) -> Result<impl Responder, HandlerError> {
    let transaction = web::block(move || {
        models::delete_transaction(&pool, user_id.into_inner(), transaction_id.into_inner())
    })
    .await
    .context("Blocking error")??;
    Ok(HttpResponse::Ok().json(transaction))
}

#[get("/categories")]
pub async fn get_all_categories(
    pool: web::Data<DbPool>,
    user_id: web::ReqData<UserId>,
) -> Result<impl Responder, HandlerError> {
    let categories = web::block(move || models::get_all_categories(&pool, user_id.into_inner()))
        .await
        .context("Blocking error")??;
    Ok(HttpResponse::Ok().json(categories))
}

#[get("/tags")]
pub async fn get_all_tags(
    pool: web::Data<DbPool>,
    user_id: web::ReqData<UserId>,
) -> Result<impl Responder, HandlerError> {
    let tags = web::block(move || models::get_all_tags(&pool, user_id.into_inner()))
        .await
        .context("Blocking error")??;
    Ok(HttpResponse::Ok().json(tags))
}

#[get("/transactees")]
pub async fn get_all_transactees(
    pool: web::Data<DbPool>,
    user_id: web::ReqData<UserId>,
) -> Result<impl Responder, HandlerError> {
    let transactees = web::block(move || models::get_all_transactees(&pool, user_id.into_inner()))
        .await
        .context("Blocking error")??;
    Ok(HttpResponse::Ok().json(transactees))
}
