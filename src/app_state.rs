use std::sync::Arc;

use crate::db::*;
use rocket::serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    pub db_path: String,
    pub key_size_hex: u16,
}

pub struct AppState {
    pub config: Config,
    pub db: Arc<dyn Database>,
}
