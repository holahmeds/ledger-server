use crate::sqlx_repo::SQLxRepo;
use crate::transaction_repo::TransactionRepoError::TransactionNotFound;
use crate::transaction_repo::{Filter, MonthlyTotal, PageOptions};
use crate::transaction_repo::{NewTransaction, Transaction, TransactionRepo, TransactionRepoError};
use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{query, query_as, query_scalar, Executor, PgExecutor, Postgres, QueryBuilder};
use std::collections::{HashMap, HashSet};
use tracing::instrument;

#[derive(sqlx::FromRow)]
struct TransactionEntry {
    id: i32,
    category: String,
    transactee: Option<String>,
    note: Option<String>,
    date: NaiveDate,
    amount: Decimal,
    #[allow(dead_code)]
    user_id: String,
}

struct TagEntry {
    transaction_id: i32,
    tag: String,
}

#[derive(sqlx::FromRow)]
struct MonthlyTotalResult {
    month: Option<DateTime<Utc>>,
    income: Option<Decimal>,
    expense: Option<Decimal>,
}

impl SQLxRepo {
    #[instrument(skip(executor))]
    async fn get_tags_single<'a, E>(
        executor: E,
        transaction_id: i32,
    ) -> Result<HashSet<String>, TransactionRepoError>
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
        Ok(HashSet::from_iter(tags))
    }

    #[instrument(skip(self))]
    async fn get_tags_multi(
        &self,
        user: &str,
        transaction_ids: Vec<i32>,
    ) -> Result<Vec<TagEntry>, TransactionRepoError> {
        let tags = query_as!(
            TagEntry,
            "SELECT * FROM transaction_tags WHERE transaction_id = ANY($1)",
            transaction_ids as Vec<i32>
        )
        .fetch_all(&self.pool)
        .await
        .with_context(|| format!("Unable to fetch tags for user {}", user))?;
        Ok(tags)
    }

    #[instrument(skip(transaction, tags))]
    async fn insert_tags<'a, 'e, I, E>(
        transaction: E,
        transaction_id: i32,
        tags: I,
    ) -> Result<(), TransactionRepoError>
    where
        I: Iterator<Item = &'a String>,
        E: Executor<'e, Database = Postgres>,
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
            .execute(transaction)
            .await
            .context("Unable to insert tags for new transaction")?;
        Ok(())
    }

    #[instrument(skip(transaction))]
    async fn delete_transaction_tags<'e, E>(
        transaction: E,
        transaction_id: i32,
        removed_tags: Vec<&str>,
    ) -> Result<(), TransactionRepoError>
    where
        E: Executor<'e, Database = Postgres>,
    {
        query!(
            "DELETE FROM transaction_tags WHERE transaction_id = $1 AND tag = ANY($2)",
            transaction_id,
            removed_tags as Vec<&str>
        )
        .execute(transaction)
        .await
        .with_context(|| format!("Unable to remove tags from transaction {}", transaction_id))?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_transaction_entry(
        &self,
        user: &str,
        transaction_id: i32,
    ) -> Result<Option<TransactionEntry>, TransactionRepoError> {
        let transaction_entry: Option<TransactionEntry> = query_as!(
            TransactionEntry,
            "SELECT * FROM transactions WHERE id = $1 AND user_id = $2",
            transaction_id,
            user
        )
        .fetch_optional(&self.pool)
        .await
        .with_context(|| format!("Unable to get transaction {}", transaction_id))?;
        Ok(transaction_entry)
    }

    #[instrument(skip(self))]
    async fn get_transaction_entries(
        &self,
        user: &str,
        from: Option<NaiveDate>,
        until: Option<NaiveDate>,
        category: Option<String>,
        transactee: Option<String>,
        page_options: Option<PageOptions>,
    ) -> Result<Vec<TransactionEntry>, TransactionRepoError> {
        let mut query_builder = QueryBuilder::new("SELECT * FROM transactions WHERE user_id = ");
        query_builder.push_bind(user);
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
        query_builder.push(" ORDER BY date DESC, id DESC");
        if let Some(po) = page_options {
            query_builder
                .push(" OFFSET ")
                .push_bind(po.offset)
                .push(" LIMIT ")
                .push_bind(po.limit);
        }
        let query = query_builder.build_query_as();
        let transaction_entries: Vec<TransactionEntry> = query
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("Unable to get transactions for user {}", user))?;
        Ok(transaction_entries)
    }

    #[instrument(skip(transaction))]
    async fn insert_transaction_entry<'e, E>(
        transaction: E,
        user: &str,
        new_transaction: &NewTransaction,
    ) -> Result<i32, TransactionRepoError>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let id = query_scalar!(
            "INSERT INTO transactions(category, transactee, note, date, amount, user_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
            new_transaction.category,
            new_transaction.transactee,
            new_transaction.note,
            new_transaction.date,
            new_transaction.amount,
            user
        ).fetch_one(transaction).await.context("Unable to insert transaction")?;
        Ok(id)
    }

    #[instrument(skip(transaction))]
    async fn update_transaction_entry<'e, E>(
        transaction: E,
        user: &str,
        transaction_id: i32,
        updated_transaction: &NewTransaction,
    ) -> Result<(), TransactionRepoError>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let result = query!(
            "UPDATE transactions SET category = $1, transactee = $2, note = $3, date = $4, amount = $5 WHERE user_id = $6 AND id = $7",
            updated_transaction.category,
            updated_transaction.transactee,
            updated_transaction.note,
            updated_transaction.date,
            updated_transaction.amount,
            user,
            transaction_id
        ).execute(transaction).await.with_context(|| format!("Unable to update transaction {}", transaction_id))?;
        if result.rows_affected() == 0 {
            Err(TransactionNotFound(transaction_id))
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self))]
    async fn delete_transaction_entry(
        &self,
        user: &str,
        transaction_id: i32,
    ) -> Result<TransactionEntry, TransactionRepoError> {
        let transaction_entry = query_as!(TransactionEntry, "DELETE FROM transactions WHERE user_id = $1 AND id = $2 RETURNING id, category, transactee, note, date, amount, user_id", user, transaction_id)
            .fetch_optional(&self.pool)
            .await
            .with_context(|| format!("Unable to delete transaction {}", transaction_id))?
            .ok_or(TransactionNotFound(transaction_id))?;
        Ok(transaction_entry)
    }
}

#[async_trait]
impl TransactionRepo for SQLxRepo {
    #[instrument(skip(self))]
    async fn get_transaction(
        &self,
        user: &str,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError> {
        let transaction_entry = self.get_transaction_entry(user, transaction_id).await?;
        let transaction_entry = transaction_entry.ok_or(TransactionNotFound(transaction_id))?;

        let tags = SQLxRepo::get_tags_single(&self.pool, transaction_id).await?;

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

    #[instrument(skip(self))]
    async fn get_all_transactions(
        &self,
        user: &str,
        filter: Filter,
        page_options: Option<PageOptions>,
    ) -> Result<Vec<Transaction>, TransactionRepoError> {
        let transaction_entries = self
            .get_transaction_entries(
                user,
                filter.from,
                filter.until,
                filter.category,
                filter.transactee,
                page_options,
            )
            .await?;

        let transaction_ids: Vec<i32> = transaction_entries.iter().map(|te| te.id).collect();
        let tags_entries: Vec<TagEntry> = self.get_tags_multi(user, transaction_ids).await?;

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
                HashSet::new(),
            );
            transactions.push(transaction);
            transaction_index.insert(te.id, transactions.len() - 1);
        }
        for te in tags_entries {
            let index = transaction_index
                .get(&te.transaction_id)
                .context("Tag's transaction ID does not match fetched transaction")?;
            transactions[*index].tags.insert(te.tag);
        }

        Ok(transactions)
    }

    #[instrument(skip(self, new_transaction))]
    async fn create_new_transaction(
        &self,
        user: &str,
        new_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .context("Unable to start transaction")?;
        let id = Self::insert_transaction_entry(&mut *transaction, user, &new_transaction).await?;
        SQLxRepo::insert_tags(&mut *transaction, id, new_transaction.tags.iter()).await?;
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

    #[instrument(skip(self, updated_transaction))]
    async fn update_transaction(
        &self,
        user: &str,
        transaction_id: i32,
        updated_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .context("Unable to start transaction")?;

        Self::update_transaction_entry(
            &mut *transaction,
            user,
            transaction_id,
            &updated_transaction,
        )
        .await?;

        let existing_tags = SQLxRepo::get_tags_single(&mut *transaction, transaction_id).await?;

        let new_tags = updated_transaction.tags.difference(&existing_tags);
        SQLxRepo::insert_tags(&mut *transaction, transaction_id, new_tags).await?;

        let removed_tags: Vec<&str> = existing_tags
            .difference(&updated_transaction.tags)
            .map(|t| t.as_str())
            .collect();
        Self::delete_transaction_tags(&mut *transaction, transaction_id, removed_tags).await?;

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

    #[instrument(skip(self))]
    async fn delete_transaction(
        &self,
        user: &str,
        transaction_id: i32,
    ) -> Result<Transaction, TransactionRepoError> {
        let tags = SQLxRepo::get_tags_single(&self.pool, transaction_id).await?;
        let transaction_entry = self.delete_transaction_entry(user, transaction_id).await?;

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

    #[instrument(skip(self))]
    async fn get_monthly_totals(
        &self,
        user: &str,
        filter: Filter,
    ) -> Result<Vec<MonthlyTotal>, TransactionRepoError> {
        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT DATE_TRUNC('month', date)             as month,
                   SUM(amount) FILTER (WHERE amount > 0) as income,
                   SUM(amount * -1) FILTER (WHERE amount < 0) as expense
            FROM transactions
            WHERE user_id = 
            "#,
        );
        query_builder.push_bind(user);

        if let Some(from) = filter.from {
            query_builder.push(" AND date >= ").push_bind(from);
        }
        if let Some(until) = filter.until {
            query_builder.push(" AND date <= ").push_bind(until);
        }
        if let Some(category) = filter.category {
            query_builder.push(" AND category = ").push_bind(category);
        }
        if let Some(transactee) = filter.transactee {
            query_builder
                .push(" AND transactee = ")
                .push_bind(transactee);
        }

        query_builder.push(" GROUP BY month ORDER BY month DESC");
        let query = query_builder.build_query_as();

        let monthly_totals: Vec<MonthlyTotalResult> = query
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("Unable to get monthly totals for {}", user))?;

        let monthly_totals = monthly_totals
            .into_iter()
            .map(|result| {
                MonthlyTotal::new(
                    result.month.unwrap().naive_utc().date(),
                    result.income.unwrap_or(Decimal::ZERO),
                    result.expense.unwrap_or(Decimal::ZERO),
                )
            })
            .collect();

        Ok(monthly_totals)
    }

    #[instrument(skip(self))]
    async fn get_all_categories(&self, user: &str) -> Result<Vec<String>, TransactionRepoError> {
        let categories = query_scalar!(
            "SELECT DISTINCT category FROM transactions WHERE user_id = $1",
            user
        )
        .fetch_all(&self.pool)
        .await
        .with_context(|| format!("Unable to get categories for user {}", user))?;
        Ok(categories)
    }

    #[instrument(skip(self))]
    async fn get_all_tags(&self, user: &str) -> Result<Vec<String>, TransactionRepoError> {
        let tags = query_scalar!("SELECT DISTINCT tag FROM transaction_tags WHERE transaction_id IN (SELECT id FROM transactions WHERE user_id = $1)", user)
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("Unable to get tags for user {}", user))?;
        Ok(tags)
    }

    #[instrument(skip(self))]
    async fn get_all_transactees(
        &self,
        user: &str,
        category: Option<String>,
    ) -> Result<Vec<String>, TransactionRepoError> {
        let query = if let Some(category) = category {
            query_scalar!(
                "SELECT transactees.transactee as \"transactee!\" FROM (SELECT DISTINCT transactee FROM transactions WHERE user_id = $1 AND transactee IS NOT NULL) transactees LEFT JOIN (SELECT transactee, COUNT(*) AS t_count FROM transactions WHERE user_id = $1 AND category = $2 GROUP BY transactee) AS t ON transactees.transactee = t.transactee ORDER BY COALESCE(t.t_count, 0) DESC",
                user, category
            )
        } else {
            query_scalar!(
                "SELECT transactee as \"transactee!\" FROM transactions WHERE user_id = $1 AND transactee IS NOT NULL GROUP BY transactee ORDER BY COUNT(transactee) DESC",
                user
            )
        };
        let transactees = query
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("Unable to get transactees for user {}", user))?;
        Ok(transactees)
    }

    #[instrument(skip(self))]
    async fn get_balance(&self, user: &str) -> Result<Decimal, TransactionRepoError> {
        let balance = query_scalar!(
            "SELECT SUM(amount) FROM transactions WHERE user_id = $1",
            user
        )
        .fetch_one(&self.pool)
        .await
        .with_context(|| format!("Unable to get balance for user {}", user))?;
        Ok(balance.unwrap_or(Decimal::ZERO))
    }
}
