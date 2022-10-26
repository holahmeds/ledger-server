#[macro_use]
extern crate diesel_migrations;
extern crate base64;

use actix_web::error::JsonPayloadError;
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::embed_migrations;
use lambda_web::{run_actix_on_lambda, LambdaError};
use ledger::auth::jwt::JWTAuth;
use ledger::{auth, transaction, user};
use std::env;
use tracing::{error, info, Level};

embed_migrations!();

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

    let manager: ConnectionManager<diesel::PgConnection> = ConnectionManager::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Unable to build database pool");

    info!("Running migrations");
    let connection = pool.get()?;
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout())?;

    let jwt_auth = JWTAuth::from_secret(secret);
    let bearer_auth_middleware = HttpAuthentication::bearer(auth::credentials_validator);

    let factory = move || {
        let mut auth_scope = web::scope("/auth").service(auth::handlers::get_token);
        if signups_enabled {
            auth_scope = auth_scope.service(auth::handlers::signup);
        }
        App::new()
            .app_data(jwt_auth.clone())
            .app_data(Data::new(pool.clone()))
            .wrap(ledger::tracing::create_middleware())
            .service(
                web::scope("/transactions")
                    .service(transaction::handlers::get_all_categories)
                    .service(transaction::handlers::get_all_tags)
                    .service(transaction::handlers::get_all_transactees)
                    .service(transaction::handlers::get_transaction)
                    .service(transaction::handlers::get_transactions)
                    .service(transaction::handlers::create_new_transaction)
                    .service(transaction::handlers::update_transaction)
                    .service(transaction::handlers::delete_transaction)
                    .wrap(bearer_auth_middleware.clone()),
            )
            .service(
                web::scope("/user")
                    .service(user::handlers::update_password)
                    .service(user::handlers::delete_user)
                    .wrap(bearer_auth_middleware.clone()),
            )
            .service(auth_scope)
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
