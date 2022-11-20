use crate::transaction_repo::TransactionRepoError::TransactionNotFound;
use crate::transaction_repo::{NewTransaction, Transaction, TransactionRepo, TransactionRepoError};
use anyhow::Context;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{query, query_as, query_scalar, PgExecutor, Pool, Postgres, QueryBuilder};
use std::collections::HashMap;

#[derive(sqlx::FromRow)]
struct TransactionEntry {
    id: i32,
    category: String,
    transactee: Option<String>,
    note: Option<String>,
    date: NaiveDate,
    amount: Decimal,
    user_id: String,
}

struct TagEntry {
    transaction_id: i32,
    tag: String,
}

pub struct SQLxTransactionRepo {
    pool: Pool<Postgres>,
}

impl SQLxTransactionRepo {
    pub fn new(pool: Pool<Postgres>) -> SQLxTransactionRepo {
        SQLxTransactionRepo { pool }
    }

    async fn get_tags<'a, E>(
        executor: E,
        transaction_id: i32,
    ) -> Result<Vec<String>, TransactionRepoError>
    where
        E: PgExecutor<'a>,
    {
        let tags: Vec<String> = query_scalar!(
            "SELECT tag FROM transaction_tags WHERE transaction_id = $1",
            transaction_id
        )
        .fetch_all(executor)
        .await
        .with_context(|| format!("Unable to get tags for transaction {}", transaction_id))?;
        Ok(tags)
    }

    async fn insert_tags<'a, I>(
        transaction: &mut sqlx::Transaction<'_, Postgres>,
        transaction_id: i32,
        tags: I,
    ) -> Result<(), TransactionRepoError>
    where
        I: Iterator<Item = &'a String>,
    {
        let mut query_builder =
            QueryBuilder::new("INSERT INTO transaction_tags(transaction_id, tag)");
        let mut empty = true;
        query_builder.push_values(tags, |mut b, t| {
            b.push_bind(transaction_id).push_bind(t);
            empty = false;
        });
        if empty {
            return Ok(());
        }

        let query = query_builder.build();
        query
            .execute(&mut *transaction)
            .await
            .context("Unable to insert tags for new transaction")?;
        Ok(())
    }
}

#[async_trait]
impl TransactionRepo for SQLxTransactionRepo {
    async fn get_transaction(
        &self,
        user: String,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError> {
        let transaction_entry: Option<TransactionEntry> = query_as!(
            TransactionEntry,
            "SELECT * FROM transactions WHERE id = $1 AND user_id = $2",
            transaction_id,
            user
        )
        .fetch_optional(&self.pool)
        .await
        .with_context(|| format!("Unable to get transaction {}", transaction_id))?;
        let transaction_entry = transaction_entry.ok_or(TransactionNotFound(transaction_id))?;

        let tags = SQLxTransactionRepo::get_tags(&self.pool, transaction_id).await?;

        Ok(Transaction::new(
            transaction_entry.id,
            transaction_entry.category,
            transaction_entry.transactee,
            transaction_entry.note,
            transaction_entry.date,
            transaction_entry.amount,
            tags,
        ))
    }

    async fn get_all_transactions(
        &self,
        user: String,
        from: Option<NaiveDate>,
        until: Option<NaiveDate>,
        category: Option<String>,
        transactee: Option<String>,
    ) -> Result<Vec<Transaction>, TransactionRepoError> {
        let mut query_builder = QueryBuilder::new("SELECT * FROM transactions WHERE user_id = ");
        query_builder.push_bind(&user);
        if let Some(from) = from {
            query_builder.push(" AND date >= ").push_bind(from);
        }
        if let Some(until) = until {
            query_builder.push(" AND date <= ").push_bind(until);
        }
        if let Some(category) = category {
            query_builder.push(" AND category = ").push_bind(category);
        }
        if let Some(transactee) = transactee {
            query_builder
                .push(" AND transactee = ")
                .push_bind(transactee);
        }
        query_builder.push(" ORDER BY date DESC");
        let query = query_builder.build_query_as();
        let transaction_entries: Vec<TransactionEntry> = query
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("Unable to get transactions for user {}", user))?;

        let transaction_ids: Vec<i32> = transaction_entries.iter().map(|te| te.id).collect();
        let tags_entries: Vec<TagEntry> = query_as!(
            TagEntry,
            "SELECT * FROM transaction_tags WHERE transaction_id = ANY($1)",
            transaction_ids as Vec<i32>
        )
        .fetch_all(&self.pool)
        .await
        .with_context(|| format!("Unable to fetch tags for user {}", user))?;

        let mut transactions: Vec<Transaction> = vec![];
        let mut transaction_index = HashMap::new();
        for te in transaction_entries {
            let transaction = Transaction::new(
                te.id,
                te.category,
                te.transactee,
                te.note,
                te.date,
                te.amount,
                vec![],
            );
            transactions.push(transaction);
            transaction_index.insert(te.id, transactions.len() - 1);
        }
        for te in tags_entries {
            let index = transaction_index
                .get(&te.transaction_id)
                .context("Tag's transaction ID does not match fetched transaction")?;
            transactions[*index].tags.push(te.tag)
        }

        Ok(transactions)
    }

    async fn create_new_transaction(
        &self,
        user: String,
        new_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .context("Unable to start transaction")?;
        let id = query_scalar!(
            "INSERT INTO transactions(category, transactee, note, date, amount, user_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
            new_transaction.category,
            new_transaction.transactee,
            new_transaction.note,
            new_transaction.date,
            new_transaction.amount,
            user
        ).fetch_one(&mut transaction).await.context("Unable to insert transaction")?;
        SQLxTransactionRepo::insert_tags(&mut transaction, id, new_transaction.tags.iter()).await?;
        transaction.commit().await.context("Transaction failed")?;

        Ok(Transaction::new(
            id,
            new_transaction.category,
            new_transaction.transactee,
            new_transaction.note,
            new_transaction.date,
            new_transaction.amount,
            new_transaction.tags,
        ))
    }

    async fn update_transaction(
        &self,
        user: String,
        transaction_id: i32,
        updated_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .context("Unable to start transaction")?;

        let result = query!(
            "UPDATE transactions SET category = $1, transactee = $2, note = $3, date = $4, amount = $5 WHERE user_id = $6 AND id = $7",
            updated_transaction.category,
            updated_transaction.transactee,
            updated_transaction.note,
            updated_transaction.date,
            updated_transaction.amount,
            user,
            transaction_id
        ).execute(&mut transaction).await.with_context(|| format!("Unable to update transaction {}", transaction_id))?;
        if result.rows_affected() == 0 {
            return Err(TransactionNotFound(transaction_id));
        }

        let existing_tags = SQLxTransactionRepo::get_tags(&mut transaction, transaction_id).await?;

        let new_tags = updated_transaction
            .tags
            .iter()
            .filter(|t| !existing_tags.contains(t));
        SQLxTransactionRepo::insert_tags(&mut transaction, transaction_id, new_tags).await?;

        let removed_tags: Vec<&str> = existing_tags
            .iter()
            .filter(|t| !updated_transaction.tags.contains(t))
            .map(|t| t.as_str())
            .collect();
        query!(
            "DELETE FROM transaction_tags WHERE transaction_id = $1 AND tag = ANY($2)",
            transaction_id,
            removed_tags as Vec<&str>
        )
        .execute(&mut transaction)
        .await
        .with_context(|| format!("Unable to remove tags from transaction {}", transaction_id))?;

        transaction
            .commit()
            .await
            .context("Unable to commit transaction")?;

        Ok(Transaction::new(
            transaction_id,
            updated_transaction.category,
            updated_transaction.transactee,
            updated_transaction.note,
            updated_transaction.date,
            updated_transaction.amount,
            updated_transaction.tags,
        ))
    }

    async fn delete_transaction(
        &self,
        user: String,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError> {
        let tags = SQLxTransactionRepo::get_tags(&self.pool, transaction_id).await?;
        let transaction_entry = query_as!(TransactionEntry, "DELETE FROM transactions WHERE user_id = $1 AND id = $2 RETURNING id, category, transactee, note, date, amount, user_id", user, transaction_id)
            .fetch_optional(&self.pool)
            .await
            .with_context(|| format!("Unable to delete transaction {}", transaction_id))?
            .ok_or(TransactionNotFound(transaction_id))?;

        Ok(Transaction::new(
            transaction_id,
            transaction_entry.category,
            transaction_entry.transactee,
            transaction_entry.note,
            transaction_entry.date,
            transaction_entry.amount,
            tags,
        ))
    }

    async fn get_all_categories(&self, user: String) -> Result<Vec<String>, TransactionRepoError> {
        let categories = query_scalar!(
            "SELECT DISTINCT category FROM transactions WHERE user_id = $1",
            user
        )
        .fetch_all(&self.pool)
        .await
        .with_context(|| format!("Unable to get categories for user {}", user))?;
        Ok(categories)
    }

    async fn get_all_tags(&self, user: String) -> Result<Vec<String>, TransactionRepoError> {
        let tags = query_scalar!("SELECT DISTINCT tag FROM transaction_tags WHERE transaction_id IN (SELECT id FROM transactions WHERE user_id = $1)", user)
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("Unable to get tags for user {}", user))?;
        Ok(tags)
    }

    async fn get_all_transactees(&self, user: String) -> Result<Vec<String>, TransactionRepoError> {
        let transactees = query_scalar!(
            "SELECT DISTINCT transactee as \"transactee!\" FROM transactions WHERE user_id = $1 AND transactee IS NOT NULL",
            user
        )
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("Unable to get transactees for user {}", user))?;
        Ok(transactees)
    }
}
