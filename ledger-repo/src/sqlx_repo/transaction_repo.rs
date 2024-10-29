use crate::sqlx_repo::SQLxRepo;
use crate::transaction_repo::TransactionRepoError::TransactionNotFound;
use crate::transaction_repo::{Filter, MonthlyTotal, PageOptions};
use crate::transaction_repo::{NewTransaction, Transaction, TransactionRepo, TransactionRepoError};
use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{query, query_as, query_scalar, Executor, Postgres, QueryBuilder};
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
    tags: Vec<String>,
}

impl From<TransactionEntry> for Transaction {
    fn from(value: TransactionEntry) -> Self {
        Transaction::new(
            value.id,
            value.category,
            value.transactee,
            value.note,
            value.date,
            value.amount,
            value.tags.into_iter().collect(),
        )
    }
}

#[derive(sqlx::FromRow)]
struct MonthlyTotalResult {
    month: Option<DateTime<Utc>>,
    income: Option<Decimal>,
    expense: Option<Decimal>,
}

impl SQLxRepo {
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

    #[instrument(skip(db_executor))]
    async fn insert_transaction_entry<'e, E>(
        db_executor: E,
        user: &str,
        new_transaction: &NewTransaction,
    ) -> Result<i32, TransactionRepoError>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let tags: Vec<String> = new_transaction.tags.iter().cloned().collect();
        let id = query_scalar!(
            "INSERT INTO transactions(category, transactee, note, date, amount, user_id, tags) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            new_transaction.category,
            new_transaction.transactee,
            new_transaction.note,
            new_transaction.date,
            new_transaction.amount,
            user,
            tags.as_slice(),
        ).fetch_one(db_executor).await.context("Unable to insert transaction")?;
        Ok(id)
    }

    #[instrument(skip(db_executor))]
    async fn update_transaction_entry<'e, E>(
        db_executor: E,
        user: &str,
        transaction_id: i32,
        updated_transaction: &NewTransaction,
    ) -> Result<(), TransactionRepoError>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let tags: Vec<String> = updated_transaction.tags.iter().cloned().collect();
        let result = query!(
            "UPDATE transactions SET category = $1, transactee = $2, note = $3, date = $4, amount = $5, tags = $6 WHERE user_id = $7 AND id = $8",
            updated_transaction.category,
            updated_transaction.transactee,
            updated_transaction.note,
            updated_transaction.date,
            updated_transaction.amount,
            tags.as_slice(),
            user,
            transaction_id
        ).execute(db_executor).await.with_context(|| format!("Unable to update transaction {}", transaction_id))?;
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
        let transaction_entry = query_as!(TransactionEntry, "DELETE FROM transactions WHERE user_id = $1 AND id = $2 RETURNING id, category, transactee, note, date, amount, user_id, tags", user, transaction_id)
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
        self.get_transaction_entry(user, transaction_id)
            .await?
            .ok_or(TransactionNotFound(transaction_id))
            .map(|t| t.into())
    }

    #[instrument(skip(self))]
    async fn get_all_transactions(
        &self,
        user: &str,
        filter: Filter,
        page_options: Option<PageOptions>,
    ) -> Result<Vec<Transaction>, TransactionRepoError> {
        let transactions = self
            .get_transaction_entries(
                user,
                filter.from,
                filter.until,
                filter.category,
                filter.transactee,
                page_options,
            )
            .await?
            .into_iter()
            .map(|transaction_entry| transaction_entry.into())
            .collect();

        Ok(transactions)
    }

    #[instrument(skip(self, new_transaction))]
    async fn create_new_transaction(
        &self,
        user: &str,
        new_transaction: NewTransaction,
    ) -> Result<Transaction, TransactionRepoError> {
        let id = Self::insert_transaction_entry(&self.pool, user, &new_transaction).await?;

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
        Self::update_transaction_entry(&self.pool, user, transaction_id, &updated_transaction)
            .await?;

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
        self.delete_transaction_entry(user, transaction_id)
            .await
            .map(|transaction_entry| transaction_entry.into())
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
        let tags = query_scalar!(
            "SELECT DISTINCT UNNEST(tags) FROM transactions WHERE user_id = $1",
            user
        )
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("Unable to get tags for user {}", user))?
            .into_iter()
            .filter_map(|t| t)
            .collect();

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
