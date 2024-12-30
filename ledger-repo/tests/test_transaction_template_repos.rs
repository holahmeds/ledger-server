mod utils;

use ledger_repo::transaction_template_repo::TransactionTemplateRepoError;
use rstest::rstest;
use utils::generator::NewTemplateGenerator;
use utils::test_user::TestUser;
use utils::RepoType;

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
