use crate::transaction_repo::TransactionRepo;
use crate::transaction_template_repo::TransactionTemplateRepo;
use crate::user_repo::UserRepo;
use std::sync::Arc;

mod transaction_repo;
mod transaction_template_repo;
mod user_repo;

pub fn create_repos() -> (
    Arc<dyn UserRepo>,
    Arc<dyn TransactionRepo>,
    Arc<dyn TransactionTemplateRepo>,
) {
    let user_repo = user_repo::MemUserRepo::new();
    let transaction_repo = transaction_repo::MemTransactionRepo::new();
    let transaction_template_repo = transaction_template_repo::MemTransactionTemplateRepo::new();

    (
        Arc::new(user_repo),
        Arc::new(transaction_repo),
        Arc::new(transaction_template_repo),
    )
}
