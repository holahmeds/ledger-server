#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate diesel;

use diesel::r2d2::ConnectionManager;
use diesel::{r2d2, PgConnection};

pub mod auth;
mod schema;
pub mod transaction;
pub mod user;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
