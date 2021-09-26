#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate diesel;

use diesel::{PgConnection, r2d2};
use diesel::r2d2::ConnectionManager;

pub mod auth;
pub mod models;
mod schema;
pub mod transaction_handlers;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
