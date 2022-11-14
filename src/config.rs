use serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    pub db_path: String,

    /// A validator might try to get a require that is not yet put
    /// by the oracle. This option configures the maximum time (in ms)
    /// that the oracle is expected to be late with the put operation.
    pub max_expected_oracle_delay_ms: u64,
}

impl Config {
    /// Size of a hex-encoded key. For actual byte size, divide by 2.
    pub const HEX_KEY_SIZE: usize = 64;
}
