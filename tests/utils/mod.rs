use std::env;

use actix_web::dev::{Body, ServiceResponse};
use actix_web::web::BytesMut;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use futures_util::StreamExt;
use serde::de::DeserializeOwned;

pub fn database_pool() -> Pool<ConnectionManager<PgConnection>> {
    let database_url = env::var("DATABASE_TEST_URL")
        .expect("DATABASE_TEST_URL not found in environment variables");
    let manager: ConnectionManager<diesel::PgConnection> = ConnectionManager::new(database_url);

    Pool::builder().build(manager).unwrap()
}

pub async fn map_body<T>(input: &mut ServiceResponse<Body>) -> T
    where
        T: DeserializeOwned,
{
    let mut body = input.take_body();
    let mut bytes = BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }

    println!("{:?}", bytes);

    let result: T = serde_json::from_slice(&bytes).unwrap();
    return result;
}
