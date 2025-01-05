use crate::sqlx_repo::SQLxRepo;
use crate::transaction_template_repo::{
    NewTransactionTemplate, TransactionTemplate, TransactionTemplateRepo,
    TransactionTemplateRepoError,
};
use anyhow::Context;
use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::{query, query_as, query_scalar};

struct TransactionTemplateEntry {
    template_id: i32,
    #[allow(dead_code)]
    user_id: String,
    name: String,
    category: Option<String>,
    transactee: Option<String>,
    note: Option<String>,
    amount: Option<Decimal>,
    tags: Vec<String>,
}

impl Into<TransactionTemplate> for TransactionTemplateEntry {
    fn into(self) -> TransactionTemplate {
        let tags = self.tags.into_iter().collect();
        TransactionTemplate {
            template_id: self.template_id,
            name: self.name,
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
            "INSERT INTO transaction_templates(category, transactee, note, amount, user_id, tags, name) VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING template_id",
            new_template.category,
            new_template.transactee,
            new_template.note,
            new_template.amount,
            user_id,
            tags.as_slice(),
            new_template.name
        ).fetch_one(&self.pool).await.context("Unable to insert template")?;

        Ok(new_template.to_transaction_template(template_id))
    }

    async fn update_template(
        &self,
        user_id: &str,
        template_id: i32,
        template: NewTransactionTemplate,
    ) -> Result<TransactionTemplate, TransactionTemplateRepoError> {
        let tags: Vec<String> = template.tags.iter().cloned().collect();

        let result = query!(
            "UPDATE transaction_templates SET category = $1, transactee = $2, note = $3, amount = $4, tags = $5, name = $6 WHERE template_id = $7 and user_id = $8",
            template.category,
            template.transactee,
            template.note,
            template.amount,
            tags.as_slice(),
            template.name,
            template_id,
            user_id,
        ).execute(&self.pool).await.context("Unable to update template")?;

        if result.rows_affected() == 0 {
            return Err(TransactionTemplateRepoError::TemplateNotFound(template_id));
        }

        Ok(template.to_transaction_template(template_id))
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
            "DELETE FROM transaction_templates WHERE user_id = $1 AND template_id = $2 returning template_id, category, transactee, note, amount, user_id, tags, name",
            user_id, template_id)
            .fetch_optional(&self.pool)
            .await
            .context("Unable to delete template")?;

        template_entry
            .map(|t| t.into())
            .ok_or(TransactionTemplateRepoError::TemplateNotFound(template_id))
    }
}
