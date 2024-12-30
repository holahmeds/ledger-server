use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
use ledger_repo::user_repo::{User, UserRepo};

#[allow(dead_code)]
pub struct TestUser {
    pub id: String,
    repo: Arc<dyn UserRepo>,
}

#[allow(dead_code)]
impl TestUser {
    pub async fn new(user_repo: &Arc<dyn UserRepo>) -> TestUser {
        let user_id = "test-user-".to_owned() + &Uuid::new_v4().to_string();
        let user = User::new(user_id.clone(), "not a real hash".to_owned());
        user_repo.create_user(user).await.unwrap();
        info!(%user_id, "Created user");
        TestUser {
            id: user_id,
            repo: user_repo.clone(),
        }
    }

    pub async fn delete(&self) {
        self.repo.delete_user(&self.id).await.unwrap()
    }
}
