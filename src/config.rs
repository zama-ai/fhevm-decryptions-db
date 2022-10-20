use serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    pub db_path: String,

    /// If a key is non-existent, time to sleep (in ms) before retrying again.
    /// Applies to GET requests.
    pub get_sleep_period_ms: u64,

    /// How many times to retry before returning NotFound on GET requests only.
    /// Set to 0 to turn off retries and make 1 attempt only.
    pub get_retry_count: u64
}

impl Config {
    /// Size of a hex-encoded key. For actual byte size, divide by 2.
    pub const HEX_KEY_SIZE: usize = 64;
}
