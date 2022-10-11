#[macro_use]
extern crate rocket;

mod config;
mod db;
mod rocksdb_backend;
mod routes;

use config::*;
use db::*;
use rocksdb_backend::*;
use routes::*;
use std::sync::Arc;

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();
    let figment = rocket.figment();

    let config: Config = figment.extract().expect("config");
    let db: Arc<dyn Database> = Arc::new(RocksDB::open(&config.db_path).expect("db open"));

    rocket
        .manage(db)
        .mount("/", routes![put_require, get_require])
}
