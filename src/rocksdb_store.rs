use std::error::Error;

use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, Options, DB};

/// A wrapper around RocksDB.
///
/// Supports put and get operations only that can be called concurrently from multiple threads.
/// Stores data in a dedicated column family.
pub struct RocksDBStore {
    db: DB,
}

impl RocksDBStore {
    pub fn open(path: &str) -> Result<Self, Box<dyn Error>> {
        let requires_cf_desc = ColumnFamilyDescriptor::new(Self::REQUIRES_CF, Options::default());

        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let db = DB::open_cf_descriptors(&db_opts, path, vec![requires_cf_desc])?;
        Ok(RocksDBStore { db })
    }

    pub fn put_require(
        &self,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let res = self.db.put_cf(self.requires_cf_handle(), key, value)?;
        Ok(res)
    }

    pub fn get_require(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Box<dyn Error + Sync + Send>> {
        let res = self.db.get_cf(self.requires_cf_handle(), key)?;
        Ok(res)
    }
}

impl RocksDBStore {
    const REQUIRES_CF: &'static str = "requires";

    fn requires_cf_handle(&self) -> &ColumnFamily {
        self.db
            .cf_handle(Self::REQUIRES_CF)
            .expect("requires CF handle")
    }
}
