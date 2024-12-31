use crate::transaction_template_repo::{
    NewTransactionTemplate, TransactionTemplate, TransactionTemplateRepo,
    TransactionTemplateRepoError,
};
use anyhow::anyhow;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

struct State {
    templates: HashMap<i32, TransactionTemplate>,
    user_templates: HashMap<String, HashSet<i32>>,
    next_id: i32,
}

pub struct MemTransactionTemplateRepo {
    state: RwLock<State>,
}

impl MemTransactionTemplateRepo {
    pub fn new() -> Self {
        let state = State {
            templates: HashMap::new(),
            user_templates: HashMap::new(),
            next_id: 0,
        };
        MemTransactionTemplateRepo {
            state: RwLock::new(state),
        }
    }

    fn read_lock(&self) -> Result<RwLockReadGuard<State>, anyhow::Error> {
        self.state
            .read()
            .map_err(|_| anyhow!("Unable to acquire lock"))
    }

    fn write_lock(&self) -> Result<RwLockWriteGuard<State>, anyhow::Error> {
        self.state
            .write()
            .map_err(|_| anyhow!("Unable to acquire lock"))
    }
}

#[async_trait]
impl TransactionTemplateRepo for MemTransactionTemplateRepo {
    async fn create_template(
        &self,
        user_id: &str,
        new_template: NewTransactionTemplate,
    ) -> Result<TransactionTemplate, TransactionTemplateRepoError> {
        let mut write_guard = self.write_lock()?;

        let id = write_guard.next_id;
        write_guard.next_id += 1;

        let template = new_template.to_transaction_template(id);

        write_guard.templates.insert(id, template.clone());
        write_guard
            .user_templates
            .entry(user_id.to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        Ok(template)
    }

    async fn update_template(
        &self,
        user_id: &str,
        template_id: i32,
        template: NewTransactionTemplate,
    ) -> Result<TransactionTemplate, TransactionTemplateRepoError> {
        let mut write_guard = self.write_lock()?;

        let Some(template_ids) = write_guard.user_templates.get_mut(user_id) else {
            return Err(TransactionTemplateRepoError::TemplateNotFound(template_id));
        };
        if !template_ids.contains(&template_id) {
            return Err(TransactionTemplateRepoError::TemplateNotFound(template_id));
        }

        let template = template.to_transaction_template(template_id);
        write_guard.templates.insert(template_id, template.clone());

        Ok(template)
    }

    async fn get_templates(
        &self,
        user_id: &str,
    ) -> Result<Vec<TransactionTemplate>, TransactionTemplateRepoError> {
        let read_guard = self.read_lock()?;

        let Some(template_ids) = read_guard.user_templates.get(user_id) else {
            return Ok(Vec::new());
        };

        let templates = template_ids
            .into_iter()
            .map(|id| {
                read_guard
                    .templates
                    .get(id)
                    .expect("templates should have all the ids in user_templates")
            })
            .cloned()
            .collect();

        Ok(templates)
    }

    async fn delete_template(
        &self,
        user_id: &str,
        template_id: i32,
    ) -> Result<TransactionTemplate, TransactionTemplateRepoError> {
        let mut write_guard = self.write_lock()?;

        let Some(template_ids) = write_guard.user_templates.get_mut(user_id) else {
            return Err(TransactionTemplateRepoError::TemplateNotFound(template_id));
        };
        if !template_ids.remove(&template_id) {
            return Err(TransactionTemplateRepoError::TemplateNotFound(template_id));
        }

        let template = write_guard
            .templates
            .remove(&template_id)
            .expect("template should exist if there is an entry in user_templates");
        Ok(template)
    }
}
