pub use std::error::Error;

pub trait Database: Sync + Send {
    fn put_require(&self, key: &[u8], value: &[u8]) -> Result<(), Box<dyn Error + Sync + Send>>;
    fn get_require(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Box<dyn Error + Sync + Send>>;
}
