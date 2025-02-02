mod utils;

use ledger_repo::user_repo::{User, UserRepoError};
use rstest::rstest;
use utils::RepoType;
use uuid::Uuid;

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_create_and_get_user(#[case] repo_type: RepoType) {
    let (user_repo, _transaction_repo, _transaction_template_repo) =
        utils::build_repos(repo_type).await;

    let user = User::new(
        "test-user-".to_owned() + &Uuid::new_v4().to_string(),
        "not a real hash".to_owned(),
    );
    user_repo.create_user(user.clone()).await.unwrap();

    let inserted_user = user_repo.get_user(&user.id).await.unwrap();

    assert_eq!(user, inserted_user);
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_create_existing_user(#[case] repo_type: RepoType) {
    let (user_repo, _transaction_repo, _transaction_template_repo) =
        utils::build_repos(repo_type).await;

    let user = User::new(
        "test-user-".to_owned() + &Uuid::new_v4().to_string(),
        "not a real hash".to_owned(),
    );
    user_repo.create_user(user.clone()).await.unwrap();

    let create_result = user_repo.create_user(user.clone()).await;
    assert!(matches!(
        create_result,
        Err(UserRepoError::UserAlreadyExists(_))
    ))
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_update_password(#[case] repo_type: RepoType) {
    let (user_repo, _transaction_repo, _transaction_template_repo) =
        utils::build_repos(repo_type).await;

    let user = User::new(
        "test-user-".to_owned() + &Uuid::new_v4().to_string(),
        "not a real hash".to_owned(),
    );
    user_repo.create_user(user.clone()).await.unwrap();

    let update_result = user_repo.update_password_hash(&user.id, "new hash").await;
    assert!(update_result.is_ok());

    let stored_user = user_repo.get_user(&user.id).await.unwrap();
    assert_eq!(user.id, stored_user.id);
    assert_ne!(user.password_hash, stored_user.password_hash);
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_update_password_invalid_user(#[case] repo_type: RepoType) {
    let (user_repo, _transaction_repo, _transaction_template_repo) =
        utils::build_repos(repo_type).await;

    let update_result = user_repo
        .update_password_hash("invalid user", "new hash")
        .await;
    assert!(matches!(update_result, Err(UserRepoError::UserNotFound(_))))
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_delete_user(#[case] repo_type: RepoType) {
    let (user_repo, _transaction_repo, _transaction_template_repo) =
        utils::build_repos(repo_type).await;

    let user = User::new(
        "test-user-".to_owned() + &Uuid::new_v4().to_string(),
        "not a real hash".to_owned(),
    );
    user_repo.create_user(user.clone()).await.unwrap();

    let delete_result = user_repo.delete_user(&user.id).await;
    assert!(delete_result.is_ok());

    let get_result = user_repo.get_user(&user.id).await;
    assert!(get_result.is_err());
}

#[rstest]
#[case::sqlx(RepoType::SQLx)]
#[case::mem(RepoType::Mem)]
#[actix_rt::test]
async fn test_delete_invalid_user(#[case] repo_type: RepoType) {
    let (user_repo, _transaction_repo, _transaction_template_repo) =
        utils::build_repos(repo_type).await;

    let delete_result = user_repo.delete_user("test-user").await;
    assert!(matches!(delete_result, Err(UserRepoError::UserNotFound(_))))
}
