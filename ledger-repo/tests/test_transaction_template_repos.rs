mod transaction_utils;
mod utils;

use fake::faker::lorem::en::Sentence;
use fake::faker::name::en::Name;
use ledger_repo::transaction_template_repo::{
    NewTransactionTemplate, TransactionTemplateRepoError,
};
use rstest::rstest;
use rust_decimal::Decimal;
use std::collections::HashSet;
use transaction_utils::test_user::TestUser;
use transaction_utils::Generator;
use transaction_utils::{FakeAmount, FakeGenerator, FakeTags, RandomSample};
use utils::RepoType;

pub struct NewTemplateGenerator {
    cat_gen: Box<dyn Generator<Option<String>>>,
    tran_gen: Box<dyn Generator<Option<String>>>,
    note_gen: Box<dyn Generator<Option<String>>>,
    amnt_gen: Box<dyn Generator<Option<Decimal>>>,
    tag_gen: Box<dyn Generator<HashSet<String>>>,
}

impl NewTemplateGenerator {
    pub fn generate(&mut self) -> NewTransactionTemplate {
        NewTransactionTemplate::new(
            self.cat_gen.gen(),
            self.tran_gen.gen(),
            self.amnt_gen.gen(),
            self.note_gen.gen(),
            self.tag_gen.gen(),
        )
    }
}

impl Default for NewTemplateGenerator {
    fn default() -> Self {
        NewTemplateGenerator {
            cat_gen: RandomSample::boxed(vec![
                None,
                Some("Misc".to_string()),
                Some("Groceries".to_string()),
                Some("Eating Out".to_string()),
                Some("Transportation".to_string()),
            ]),
            tran_gen: FakeGenerator::boxed(Name()),
            note_gen: FakeGenerator::boxed(Sentence(5..10)),
            amnt_gen: Box::new(FakeAmount),
            tag_gen: Box::new(FakeTags),
        }
    }
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_create_and_get_templates(#[case] repo_type: RepoType) {
    let (user_repo, _transaction_repo, transaction_template_repo) =
        utils::build_repos(repo_type).await;
    let user = TestUser::new(&user_repo).await;

    let mut generator = NewTemplateGenerator::default();

    let new_template = generator.generate();

    transaction_template_repo
        .create_template(&user.id, new_template.clone())
        .await
        .unwrap();

    let templates = transaction_template_repo
        .get_templates(&user.id)
        .await
        .unwrap();

    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].category, new_template.category);
    assert_eq!(templates[0].transactee, new_template.transactee);
    assert_eq!(templates[0].amount, new_template.amount);
    assert_eq!(templates[0].note, new_template.note);
    assert_eq!(templates[0].tags, new_template.tags);

    user.delete().await;
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_get_invalid_user(#[case] repo_type: RepoType) {
    let (user_repo, _transaction_repo, transaction_template_repo) =
        utils::build_repos(repo_type).await;
    let user1 = TestUser::new(&user_repo).await;

    let mut generator = NewTemplateGenerator::default();

    let new_template = generator.generate();

    transaction_template_repo
        .create_template(&user1.id, new_template.clone())
        .await
        .unwrap();

    let result = transaction_template_repo
        .get_templates("invalid_user")
        .await
        .unwrap();
    assert!(result.is_empty());

    user1.delete().await;
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_delete_template(#[case] repo_type: RepoType) {
    let (user_repo, _transaction_repo, transaction_template_repo) =
        utils::build_repos(repo_type).await;
    let user = TestUser::new(&user_repo).await;

    let mut generator = NewTemplateGenerator::default();

    let new_template = generator.generate();
    let template = transaction_template_repo
        .create_template(&user.id, new_template.clone())
        .await
        .unwrap();

    let result = transaction_template_repo
        .delete_template(&user.id, template.template_id)
        .await;
    assert!(result.is_ok());

    let result = transaction_template_repo
        .delete_template(&user.id, template.template_id)
        .await;
    assert!(matches!(
        result,
        Err(TransactionTemplateRepoError::TemplateNotFound(_))
    ));

    user.delete().await;
}
