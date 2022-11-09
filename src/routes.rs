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

/// A require that is received/sent as JSON over HTTP.
#[derive(Serialize, Deserialize)]
pub struct Require {
    pub value: bool,
    pub signature: String,
}

// A require that is stored in the DB or in cache.
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredRequire {
    value: bool,
    signature: Vec<u8>,
}

impl TryFrom<Require> for StoredRequire {
    type Error = Status;

    fn try_from(require: Require) -> Result<Self, Self::Error> {
        let signature = base64_decode(&require.signature)?;
        Ok(StoredRequire {
            value: require.value,
            signature,
        })
    }
}

impl From<StoredRequire> for Require {
    fn from(stored_require: StoredRequire) -> Require {
        Require {
            value: stored_require.value,
            signature: base64::encode(stored_require.signature),
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

#[put("/require/<key>", data = "<require>")]
pub async fn put_require(
    db: &State<Arc<RocksDBStore>>,
    cache: &State<Arc<WaitCache<Vec<u8>, StoredRequire>>>,
    key: Key,
    require: Json<Require>,
) -> Result<(), Status> {
    let stored_require = StoredRequire::try_from(require.0)?;
    let key_clone = key.0.clone();
    let stored_require_bytes = bincode_serialize(&stored_require)?;
    let db = db.inner().clone();
    spawn_blocking(move || db.put_require(&key_clone, &stored_require_bytes))
        .await
        .map_err(|_| Status::ServiceUnavailable)?
        .map_err(|_| Status::InternalServerError)?;
    cache.put(key.0, stored_require);
    Ok(())
}

#[get("/require/<key>")]
pub async fn get_require(
    config: &State<Config>,
    db: &State<Arc<RocksDBStore>>,
    cache: &State<Arc<WaitCache<Vec<u8>, StoredRequire>>>,
    key: Key,
) -> Result<Json<Require>, Status> {
    let key_clone = key.0.clone();
    let db = db.inner().clone();
    let value = spawn_blocking(move || db.get_require(&key_clone))
        .await
        .map_err(|_| Status::ServiceUnavailable)?
        .map_err(|_| Status::InternalServerError)?;
    if let Some(value) = value {
        let stored_require: StoredRequire = bincode_deserialize(&value)?;
        Ok(Json(Require::from(stored_require)))
    } else if let Some(stored_require) = cache
        .get_timeout(
            key.0,
            Duration::from_millis(config.max_expected_oracle_delay_ms),
        )
        .await
    {
        Ok(Json(Require::from(stored_require)))
    } else {
        Err(Status::NotFound)
    }
}
