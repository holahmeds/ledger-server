use crate::transaction_repo::TransactionRepo;
use crate::user_repo::UserRepo;
use std::sync::Arc;

mod transaction_repo;
mod user_repo;

pub fn create_repos() -> (Arc<dyn TransactionRepo>, Arc<dyn UserRepo>) {
    let transaction_repo = transaction_repo::MemTransactionRepo::new();
    let user_repo = user_repo::MemUserRepo::new();

    (Arc::new(transaction_repo), Arc::new(user_repo))
}
