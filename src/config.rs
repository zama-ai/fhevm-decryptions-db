use serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    pub db_path: String,
}

impl Config {
    pub const HEX_KEY_SIZE: usize = 64;
}
