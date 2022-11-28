use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;

use crate::error::HandlerError;
use crate::user::UserId;

use ledger_repo::transaction_repo::TransactionRepo;
use ledger_repo::transaction_repo::{NewTransaction, PageOptions};

#[derive(Deserialize)]
pub struct Filter {
    from: Option<NaiveDate>,
    until: Option<NaiveDate>,
    category: Option<String>,
    transactee: Option<String>,
}

#[derive(Deserialize)]
pub struct PageQueryParameters {
    page_number: Option<i64>,
    page_size: Option<i64>,
}

#[get("/{transaction_id}")]
pub async fn get_transaction(
    transaction_repo: web::Data<Arc<dyn TransactionRepo>>,
    user_id: web::ReqData<UserId>,
    transaction_id: web::Path<i32>,
) -> Result<impl Responder, HandlerError> {
    let transaction = transaction_repo
        .get_transaction(user_id.into_inner(), transaction_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(transaction))
}

#[get("")]
pub async fn get_transactions(
    transaction_repo: web::Data<Arc<dyn TransactionRepo>>,
    user_id: web::ReqData<UserId>,
    filter: web::Query<Filter>,
    page: web::Query<PageQueryParameters>,
) -> Result<impl Responder, HandlerError> {
    if page.page_number.is_some() ^ page.page_size.is_some() {
        return Err(HandlerError::BadRequest(
            "Both 'page_number' and 'page_size' is required for paging".to_string(),
        ));
    }

    let filter = filter.into_inner();
    let page_options = if page.page_size.is_some() && page.page_number.is_some() {
        Some(PageOptions {
            page_number: page.page_number.unwrap(),
            page_size: page.page_size.unwrap(),
        })
    } else {
        None
    };

    let transaction = transaction_repo
        .get_all_transactions(
            user_id.into_inner(),
            filter.from,
            filter.until,
            filter.category,
            filter.transactee,
            page_options,
        )
        .await?;
    Ok(HttpResponse::Ok().json(transaction))
}

#[post("")]
pub async fn create_new_transaction(
    transaction_repo: web::Data<Arc<dyn TransactionRepo>>,
    user_id: web::ReqData<UserId>,
    new_transaction: web::Json<NewTransaction>,
) -> Result<impl Responder, HandlerError> {
    let transaction = transaction_repo
        .create_new_transaction(user_id.into_inner(), new_transaction.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(transaction))
}

#[put("/{transaction_id}")]
pub async fn update_transaction(
    transaction_repo: web::Data<Arc<dyn TransactionRepo>>,
    user_id: web::ReqData<UserId>,
    transaction_id: web::Path<i32>,
    updated_transaction: web::Json<NewTransaction>,
) -> Result<impl Responder, HandlerError> {
    let transaction = transaction_repo
        .update_transaction(
            user_id.into_inner(),
            transaction_id.into_inner(),
            updated_transaction.into_inner(),
        )
        .await?;
    Ok(HttpResponse::Ok().json(transaction))
}

#[delete("/{transaction_id}")]
pub async fn delete_transaction(
    transaction_repo: web::Data<Arc<dyn TransactionRepo>>,
    user_id: web::ReqData<UserId>,
    transaction_id: web::Path<i32>,
) -> Result<impl Responder, HandlerError> {
    let transaction = transaction_repo
        .delete_transaction(user_id.into_inner(), transaction_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(transaction))
}

#[get("/categories")]
pub async fn get_all_categories(
    transaction_repo: web::Data<Arc<dyn TransactionRepo>>,
    user_id: web::ReqData<UserId>,
) -> Result<impl Responder, HandlerError> {
    let categories = transaction_repo
        .get_all_categories(user_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(categories))
}

#[get("/tags")]
pub async fn get_all_tags(
    transaction_repo: web::Data<Arc<dyn TransactionRepo>>,
    user_id: web::ReqData<UserId>,
) -> Result<impl Responder, HandlerError> {
    let tags = transaction_repo.get_all_tags(user_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(tags))
}

#[get("/transactees")]
pub async fn get_all_transactees(
    transaction_repo: web::Data<Arc<dyn TransactionRepo>>,
    user_id: web::ReqData<UserId>,
) -> Result<impl Responder, HandlerError> {
    let transactees = transaction_repo
        .get_all_transactees(user_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(transactees))
}
