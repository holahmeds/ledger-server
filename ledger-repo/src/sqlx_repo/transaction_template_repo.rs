use crate::sqlx_repo::SQLxRepo;
use crate::transaction_template_repo::{
    NewTransactionTemplate, TransactionTemplate, TransactionTemplateRepo,
    TransactionTemplateRepoError,
};
use anyhow::Context;
use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::{query_as, query_scalar};

struct TransactionTemplateEntry {
    template_id: i32,
    category: Option<String>,
    transactee: Option<String>,
    note: Option<String>,
    amount: Option<Decimal>,
    #[allow(dead_code)]
    user_id: String,
    tags: Vec<String>,
}

impl Into<TransactionTemplate> for TransactionTemplateEntry {
    fn into(self) -> TransactionTemplate {
        let tags = self.tags.into_iter().collect();
        TransactionTemplate {
            template_id: self.template_id,
            category: self.category,
            transactee: self.transactee,
            amount: self.amount,
            note: self.note,
            tags,
        }
    }
}

#[async_trait]
impl TransactionTemplateRepo for SQLxRepo {
    async fn create_template(
        &self,
        user_id: &str,
        new_template: NewTransactionTemplate,
    ) -> Result<TransactionTemplate, TransactionTemplateRepoError> {
        let tags: Vec<String> = new_template.tags.iter().cloned().collect();
        let template_id = query_scalar!(
            "INSERT INTO transaction_templates(category, transactee, note, amount, user_id, tags) VALUES($1, $2, $3, $4, $5, $6) RETURNING template_id",
            new_template.category,
            new_template.transactee,
            new_template.note,
            new_template.amount,
            user_id,
            tags.as_slice()
        ).fetch_one(&self.pool).await.context("Unable to insert template")?;

        Ok(new_template.to_transaction_template(template_id))
    }

    async fn get_templates(
        &self,
        user_id: &str,
    ) -> Result<Vec<TransactionTemplate>, TransactionTemplateRepoError> {
        let transaction_templates: Vec<TransactionTemplateEntry> = query_as!(
            TransactionTemplateEntry,
            "SELECT * FROM transaction_templates WHERE user_id = $1",
            user_id
        )
            .fetch_all(&self.pool)
            .await
            .context("Unable to retrieve templates")?;

        let transaction_templates = transaction_templates
            .into_iter()
            .map(|t| t.into())
            .collect();

        Ok(transaction_templates)
    }

    async fn delete_template(
        &self,
        user_id: &str,
        template_id: i32,
    ) -> Result<TransactionTemplate, TransactionTemplateRepoError> {
        let template_entry = query_as!(
            TransactionTemplateEntry,
            "DELETE FROM transaction_templates WHERE user_id = $1 AND template_id = $2 returning template_id, category, transactee, note, amount, user_id, tags",
            user_id, template_id)
            .fetch_optional(&self.pool)
            .await
            .context("Unable to delete template")?;

        template_entry
            .map(|t| t.into())
            .ok_or(TransactionTemplateRepoError::TemplateNotFound(template_id))
    }
}
