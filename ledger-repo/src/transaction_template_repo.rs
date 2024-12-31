use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionTemplate {
    pub template_id: i32,
    pub category: Option<String>,
    pub transactee: Option<String>,
    pub amount: Option<Decimal>,
    pub note: Option<String>,
    pub tags: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTransactionTemplate {
    pub category: Option<String>,
    pub transactee: Option<String>,
    pub amount: Option<Decimal>,
    pub note: Option<String>,
    pub tags: HashSet<String>,
}

impl NewTransactionTemplate {
    pub fn new(
        category: Option<String>,
        transactee: Option<String>,
        amount: Option<Decimal>,
        note: Option<String>,
        tags: HashSet<String>,
    ) -> Self {
        NewTransactionTemplate {
            category,
            transactee,
            amount,
            note,
            tags,
        }
    }

    pub fn to_transaction_template(self, template_id: i32) -> TransactionTemplate {
        TransactionTemplate {
            template_id,
            category: self.category,
            transactee: self.transactee,
            amount: self.amount,
            note: self.note,
            tags: self.tags,
        }
    }
}

#[derive(Error, Debug)]
pub enum TransactionTemplateRepoError {
    #[error("Template with id {0} not found")]
    TemplateNotFound(i32),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait TransactionTemplateRepo: Sync + Send {
    async fn create_template(
        &self,
        user_id: &str,
        new_template: NewTransactionTemplate,
    ) -> Result<TransactionTemplate, TransactionTemplateRepoError>;

    async fn update_template(
        &self,
        user_id: &str,
        template_id: i32,
        template: NewTransactionTemplate,
    ) -> Result<TransactionTemplate, TransactionTemplateRepoError>;

    async fn get_templates(
        &self,
        user_id: &str,
    ) -> Result<Vec<TransactionTemplate>, TransactionTemplateRepoError>;

    async fn delete_template(
        &self,
        user_id: &str,
        template_id: i32,
    ) -> Result<TransactionTemplate, TransactionTemplateRepoError>;
}
