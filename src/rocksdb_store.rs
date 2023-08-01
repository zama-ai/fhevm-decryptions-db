// BSD 3-Clause Clear License

// Copyright © 2023 ZAMA.
// All rights reserved.

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
        let decryptions_cf_desc =
            ColumnFamilyDescriptor::new(Self::DECRYPTIONS_CF, Options::default());

        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let db = DB::open_cf_descriptors(&db_opts, path, vec![decryptions_cf_desc])?;
        Ok(RocksDBStore { db })
    }

    pub fn put_decryption(
        &self,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        self.db.put_cf(self.decryptions_cf_handle(), key, value)?;
        Ok(())
    }

    pub fn get_decryption(
        &self,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, Box<dyn Error + Sync + Send>> {
        let res = self.db.get_cf(self.decryptions_cf_handle(), key)?;
        Ok(res)
    }
}

impl RocksDBStore {
    const DECRYPTIONS_CF: &'static str = "decryptions";

    fn decryptions_cf_handle(&self) -> &ColumnFamily {
        self.db
            .cf_handle(Self::DECRYPTIONS_CF)
            .expect("decryptions CF handle")
    }
}
