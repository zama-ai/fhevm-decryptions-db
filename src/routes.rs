// BSD 3-Clause Clear License

// Copyright © 2023 ZAMA.
// All rights reserved.

use crate::{config::*, rocksdb_store::RocksDBStore, wait_cache::WaitCache};
use rocket::{
    http::Status,
    request::FromParam,
    serde::json::Json,
    tokio::{task::spawn_blocking, time::Duration},
    State,
};
use serde::{Deserialize, Serialize};
use std::{
    convert::{From, TryFrom},
    sync::Arc,
};

fn hex_decode(input: &str) -> Result<Vec<u8>, Status> {
    hex::decode(input).map_err(|_| Status::BadRequest)
}

fn base64_decode(input: &str) -> Result<Vec<u8>, Status> {
    base64::decode(input).map_err(|_| Status::BadRequest)
}

fn bincode_serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, Status> {
    bincode::serialize(value).map_err(|_| Status::InternalServerError)
}

fn bincode_deserialize<'a, T: Deserialize<'a>>(input: &'a [u8]) -> Result<T, Status> {
    bincode::deserialize(input).map_err(|_| Status::InternalServerError)
}

/// A decryption that is received/sent as JSON over HTTP.
#[derive(Serialize, Deserialize)]
pub struct Decryption {
    pub value: u64,
    pub signature: String,
}

// A decryption that is stored in the DB or in cache.
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredDecryption {
    value: u64,
    signature: Vec<u8>,
}

impl TryFrom<Decryption> for StoredDecryption {
    type Error = Status;

    fn try_from(decryption: Decryption) -> Result<Self, Self::Error> {
        let signature = base64_decode(&decryption.signature)?;
        Ok(StoredDecryption {
            value: decryption.value,
            signature,
        })
    }
}

impl From<StoredDecryption> for Decryption {
    fn from(stored_decryption: StoredDecryption) -> Decryption {
        Decryption {
            value: stored_decryption.value,
            signature: base64::encode(stored_decryption.signature),
        }
    }
}

pub struct Key(Vec<u8>);

#[rocket::async_trait]
impl<'r> FromParam<'r> for Key {
    type Error = &'r str;

    fn from_param(key: &'r str) -> Result<Self, Self::Error> {
        if key.len() != Config::HEX_KEY_SIZE {
            return Err(key);
        }
        match hex_decode(key) {
            Ok(key) => Ok(Key(key)),
            _ => Err(key),
        }
    }
}

#[put("/decryption/<key>", data = "<decryption>")]
pub async fn put_decryption(
    db: &State<Arc<RocksDBStore>>,
    cache: &State<Arc<WaitCache<Vec<u8>, StoredDecryption>>>,
    key: Key,
    decryption: Json<Decryption>,
) -> Result<(), Status> {
    let stored_decryption = StoredDecryption::try_from(decryption.0)?;
    let key_clone = key.0.clone();
    let stored_decryption_bytes = bincode_serialize(&stored_decryption)?;
    let db = db.inner().clone();
    spawn_blocking(move || db.put_decryption(&key_clone, &stored_decryption_bytes))
        .await
        .map_err(|_| Status::ServiceUnavailable)?
        .map_err(|_| Status::InternalServerError)?;
    cache.put(key.0, stored_decryption);
    Ok(())
}

#[get("/decryption/<key>")]
pub async fn get_decryption(
    config: &State<Config>,
    db: &State<Arc<RocksDBStore>>,
    cache: &State<Arc<WaitCache<Vec<u8>, StoredDecryption>>>,
    key: Key,
) -> Result<Json<Decryption>, Status> {
    let key_clone = key.0.clone();
    let db = db.inner().clone();
    let value = spawn_blocking(move || db.get_decryption(&key_clone))
        .await
        .map_err(|_| Status::ServiceUnavailable)?
        .map_err(|_| Status::InternalServerError)?;
    if let Some(value) = value {
        let stored_decryption: StoredDecryption = bincode_deserialize(&value)?;
        Ok(Json(Decryption::from(stored_decryption)))
    } else if let Some(stored_decryption) = cache
        .get_timeout(
            key.0,
            Duration::from_millis(config.max_expected_oracle_delay_ms),
        )
        .await
    {
        Ok(Json(Decryption::from(stored_decryption)))
    } else {
        Err(Status::NotFound)
    }
}
