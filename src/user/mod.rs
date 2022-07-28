use serde::Deserialize;

pub mod handlers;
mod models;

#[derive(Deserialize)]
pub struct NewUser {
    pub id: String,
    pub password: String,
}
