// BSD 3-Clause Clear License

// Copyright © 2023 ZAMA.
// All rights reserved.

#[macro_use]
extern crate rocket;

use rocket::{fairing::AdHoc, Build, Rocket};

pub mod config;
pub mod rocksdb_store;
pub mod routes;
pub mod wait_cache;

use config::*;
use rocksdb_store::*;
use routes::*;
use std::{sync::Arc, time::Duration};
use wait_cache::WaitCache;

/// Builds a rocket instance with relevant configuration options.
pub fn build_and_configure_rocket() -> Rocket<Build> {
    let rocket = build_rocket();
    configure_rocket(rocket)
}

pub fn build_rocket() -> Rocket<Build> {
    rocket::build()
}

pub fn configure_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    let figment = rocket.figment();

    let config: Config = figment.extract().expect("config");
    let db = Arc::new(RocksDBStore::open(&config.db_path).expect("db open"));
    let cache = Arc::new(WaitCache::<Vec<u8>, StoredDecryption>::new(
        Duration::from_secs(config.max_expected_oracle_delay_ms),
    ));

    rocket
        .manage(db)
        .manage(cache)
        .mount("/", routes![put_decryption, get_decryption])
        .attach(AdHoc::config::<Config>())
}
