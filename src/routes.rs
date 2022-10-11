use crate::app_state::*;
use rocket::{http::Status, serde::json::Json, tokio::task::spawn_blocking, State};
use serde::{Deserialize, Serialize};
use std::convert::{From, TryFrom};

fn hex_decode(input: &str) -> Result<Vec<u8>, Status> {
    hex::decode(input).map_err(|_| Status::BadRequest)
}

fn bincode_serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, Status> {
    bincode::serialize(value).map_err(|_| Status::InternalServerError)
}

fn bincode_deserialize<'a, T: Deserialize<'a>>(input: &'a [u8]) -> Result<T, Status> {
    bincode::deserialize(input).map_err(|_| Status::InternalServerError)
}

fn parse_key(key: &str, key_size_hex: u16) -> Result<Vec<u8>, Status> {
    if key.len() != key_size_hex as usize {
        return Err(Status::BadRequest);
    }
    hex_decode(key).map_err(|_| Status::BadRequest)
}

#[derive(Serialize, Deserialize)]
pub struct Require {
    value: bool,
    hex_signature: String,
}

#[derive(Serialize, Deserialize)]
struct StoredRequire {
    value: bool,
    signature: Vec<u8>,
}

impl TryFrom<Require> for StoredRequire {
    type Error = Status;

    fn try_from(require: Require) -> Result<Self, Self::Error> {
        let signature = hex_decode(&require.hex_signature)?;
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
            hex_signature: hex::encode(stored_require.signature),
        }
    }
}

#[put("/require/<key>", data = "<require>")]
pub async fn put_require(
    state: &State<AppState>,
    key: &str,
    require: Json<Require>,
) -> Result<(), Status> {
    let key = parse_key(&key, state.config.key_size_hex)?;
    let stored_require = StoredRequire::try_from(require.0)?;
    let stored_require = bincode_serialize(&stored_require)?;
    let db = state.inner().db.clone();
    spawn_blocking(move || db.put_require(&key, &stored_require))
        .await
        .map_err(|_| Status::ServiceUnavailable)?
        .map_err(|_| Status::InternalServerError)?;
    Ok(())
}

#[get("/require/<key>")]
pub async fn get_require<'a>(state: &State<AppState>, key: &str) -> Result<Json<Require>, Status> {
    let key = parse_key(&key, state.config.key_size_hex)?;
    let db = state.inner().db.clone();
    let value = spawn_blocking(move || db.get_require(&key))
        .await
        .map_err(|_| Status::ServiceUnavailable)?
        .map_err(|_| Status::InternalServerError)?;
    if let Some(value) = value {
        let stored_require: StoredRequire = bincode_deserialize(&value)?;
        Ok(Json(Require::from(stored_require)))
    } else {
        Err(Status::NotFound)
    }
}
