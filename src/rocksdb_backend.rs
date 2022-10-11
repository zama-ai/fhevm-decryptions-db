use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, Options, DB};

use crate::db::*;

pub struct RocksDB {
    db: DB,
}

impl RocksDB {
    const REQUIRES_CF: &'static str = "requires";

    pub fn open(path: &str) -> Result<Self, Box<dyn Error>> {
        let requires_cf_desc = ColumnFamilyDescriptor::new(Self::REQUIRES_CF, Options::default());

        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let db = DB::open_cf_descriptors(&db_opts, path, vec![requires_cf_desc])?;
        Ok(RocksDB { db })
    }

    fn requires_cf_handle(&self) -> &ColumnFamily {
        self.db
            .cf_handle(Self::REQUIRES_CF)
            .expect("requires CF handle")
    }
}

impl Database for RocksDB {
    fn put_require(&self, key: &[u8], value: &[u8]) -> Result<(), Box<dyn Error + Sync + Send>> {
        let res = self.db.put_cf(self.requires_cf_handle(), key, value)?;
        Ok(res)
    }

    fn get_require(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Box<dyn Error + Sync + Send>> {
        let res = self.db.get_cf(self.requires_cf_handle(), key)?;
        Ok(res)
    }
}
