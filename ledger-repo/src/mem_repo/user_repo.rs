use crate::user_repo::UserRepoError::{UserAlreadyExists, UserNotFound};
use crate::user_repo::{User, UserRepo, UserRepoError};
use anyhow::anyhow;
use async_trait::async_trait;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct MemUserRepo {
    password_hash: RwLock<HashMap<String, String>>,
}

impl MemUserRepo {
    pub fn new() -> MemUserRepo {
        MemUserRepo {
            password_hash: RwLock::new(HashMap::new()),
        }
    }

    fn read_lock(&self) -> Result<RwLockReadGuard<HashMap<String, String>>, anyhow::Error> {
        self.password_hash
            .read()
            .map_err(|_| anyhow!("Unable to acquire lock"))
    }

    fn write_lock(&self) -> Result<RwLockWriteGuard<HashMap<String, String>>, anyhow::Error> {
        self.password_hash
            .write()
            .map_err(|_| anyhow!("Unable to acquire lock"))
    }
}

#[async_trait]
impl UserRepo for MemUserRepo {
    async fn get_user(&self, user_id: &str) -> Result<User, UserRepoError> {
        let read_guard = self.read_lock()?;

        if let Some(h) = read_guard.get(user_id) {
            Ok(User::new(user_id.to_string(), h.to_owned()))
        } else {
            Err(UserNotFound(user_id.to_owned()))
        }
    }

    async fn create_user(&self, user: User) -> Result<(), UserRepoError> {
        let mut write_guard = self.write_lock()?;

        match write_guard.entry(user.id.clone()) {
            Entry::Occupied(_) => Err(UserAlreadyExists(user.id)),
            Entry::Vacant(e) => {
                e.insert(user.password_hash);
                Ok(())
            }
        }
    }

    async fn update_password_hash(
        &self,
        user_id: &str,
        password_hash: &str,
    ) -> Result<(), UserRepoError> {
        let mut write_guard = self.write_lock()?;

        match write_guard.entry(user_id.to_owned()) {
            Entry::Occupied(mut e) => {
                e.insert(password_hash.to_owned());
                Ok(())
            }
            Entry::Vacant(e) => Err(UserNotFound(e.into_key())),
        }
    }

    async fn delete_user(&self, user_id: &str) -> Result<(), UserRepoError> {
        let mut write_guard = self.write_lock()?;

        if write_guard.remove(user_id).is_some() {
            Ok(())
        } else {
            Err(UserNotFound(user_id.to_owned()))
        }
    }
}
