use crate::error::HandlerError;
use crate::user::UserId;
use actix_web::{web, HttpResponse, Responder};
use ledger_repo::transaction_template_repo::{NewTransactionTemplate, TransactionTemplateRepo};
use std::sync::Arc;

#[post("")]
pub async fn create_template(
    template_repo: web::Data<Arc<dyn TransactionTemplateRepo>>,
    user_id: web::ReqData<UserId>,
    new_template: web::Json<NewTransactionTemplate>,
) -> Result<impl Responder, HandlerError> {
    let template = template_repo
        .create_template(&user_id.into_inner(), new_template.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(template))
}

#[put("/{template_id}")]
pub async fn update_template(
    template_repo: web::Data<Arc<dyn TransactionTemplateRepo>>,
    user_id: web::ReqData<UserId>,
    template_id: web::Path<i32>,
    updated_template: web::Json<NewTransactionTemplate>,
) -> Result<impl Responder, HandlerError> {
    let template = template_repo
        .update_template(
            &user_id.into_inner(),
            template_id.into_inner(),
            updated_template.into_inner(),
        )
        .await?;
    Ok(HttpResponse::Ok().json(template))
}

#[get("")]
pub async fn get_all_templates(
    template_repo: web::Data<Arc<dyn TransactionTemplateRepo>>,
    user_id: web::ReqData<UserId>,
) -> Result<impl Responder, HandlerError> {
    let templates = template_repo.get_templates(&user_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(templates))
}

#[delete("/{template_id}")]
pub async fn delete_template(
    template_repo: web::Data<Arc<dyn TransactionTemplateRepo>>,
    user_id: web::ReqData<UserId>,
    template_id: web::Path<i32>,
) -> Result<impl Responder, HandlerError> {
    let template = template_repo
        .delete_template(&user_id.into_inner(), template_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(template))
}
