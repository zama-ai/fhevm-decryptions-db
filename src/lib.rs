#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};

pub mod config;
pub mod rocksdb_store;
pub mod routes;

use config::*;
use rocksdb_store::*;
use routes::*;
use std::sync::Arc;

pub fn build_rocket() -> Rocket<Build> {
    let rocket = rocket::build();
    let figment = rocket.figment();

    let config: Config = figment.extract().expect("config");
    let db = Arc::new(RocksDBStore::open(&config.db_path).expect("db open"));

    rocket
        .manage(db)
        .mount("/", routes![put_require, get_require])
}
