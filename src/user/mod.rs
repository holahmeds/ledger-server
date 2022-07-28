use serde::Deserialize;

pub mod handlers;
pub mod models;

#[derive(Deserialize)]
pub struct NewUser {
    pub id: String,
    pub password: String,
}
