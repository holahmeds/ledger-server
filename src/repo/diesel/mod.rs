mod schema;
mod transaction_repo;
mod user_repo;

use super::transaction_repo::TransactionRepo;
use super::user_repo::UserRepo;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use std::sync::Arc;
use tracing::info;
use transaction_repo::DieselTransactionRepo;
use user_repo::DieselUserRepo;

embed_migrations!();

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

fn create_db_pool(database_url: String, max_size: u32) -> DbPool {
    let manager: ConnectionManager<PgConnection> = ConnectionManager::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(max_size)
        .build(manager)
        .expect("Unable to build database pool");
    info!("Database pool created");
    pool
}

pub fn create_repos(
    database_url: String,
    max_size: u32,
    run_migrations: bool,
) -> (Arc<dyn TransactionRepo>, Arc<dyn UserRepo>) {
    let pool = create_db_pool(database_url, max_size);

    if run_migrations {
        info!("Running migrations");
        let connection = pool.get().unwrap();
        embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();
    }

    let transaction_repo = DieselTransactionRepo::new(pool.clone());
    let user_repo = DieselUserRepo::new(pool);
    (Arc::new(transaction_repo), Arc::new(user_repo))
}
