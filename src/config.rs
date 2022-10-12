use serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    pub db_path: String,
}

impl Config {
    /// Size of a hex-encoded key. For actual byte size, divide by 2.
    pub const HEX_KEY_SIZE: usize = 64;
}
