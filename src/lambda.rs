extern crate base64;

use actix_web::error::JsonPayloadError;
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use lambda_web::{run_actix_on_lambda, LambdaError};
use ledger::auth::jwt::JWTAuth;
use ledger::repo::sqlx::create_repos;
use ledger::{auth, transaction, user};
use std::env;
use tracing::{error, info, Level};

#[actix_web::main]
async fn main() -> Result<(), LambdaError> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        // ansi escape codes messes up cloudwatch logs
        .with_ansi(false)
        // cloudwatch adds timestamps
        .without_time()
        .init();
    info!("tracing initialized");

    let signups_enabled = env::var("SIGNUPS_ENABLED")
        .expect("SIGNUPS_ENABLED not set")
        .parse()?;
    let secret = base64::decode(env::var("SECRET").expect("SECRET not set"))?;
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let (transaction_repo, user_repo) = create_repos(database_url, 1).await;

    let jwt_auth = JWTAuth::from_secret(secret);
    let bearer_auth_middleware = HttpAuthentication::bearer(auth::credentials_validator);

    let factory = move || {
        App::new()
            .app_data(jwt_auth.clone())
            .app_data(Data::new(transaction_repo.clone()))
            .app_data(Data::new(user_repo.clone()))
            .wrap(ledger::tracing::create_middleware())
            .service(transaction::transaction_service().wrap(bearer_auth_middleware.clone()))
            .service(user::user_service().wrap(bearer_auth_middleware.clone()))
            .service(auth::auth_service(signups_enabled))
            .app_data(web::JsonConfig::default().error_handler(|err, req| {
                error!(req_path = req.path(), %err);
                match err {
                    JsonPayloadError::Deserialize(deserialize_err) => {
                        let error_body = serde_json::json!({
                            "error": "Unable to parse JSON payload",
                            "detail": format!("{}", deserialize_err),
                        });
                        actix_web::error::InternalError::from_response(
                            deserialize_err,
                            HttpResponse::BadRequest()
                                .content_type("application/json")
                                .body(error_body.to_string()),
                        )
                        .into()
                    }
                    _ => err.into(),
                }
            }))
    };
    run_actix_on_lambda(factory).await?;
    Ok(())
}
