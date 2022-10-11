#[macro_use]
extern crate rocket;

use std::sync::Arc;

mod routes;
use routes::*;

mod rocksdb_backend;
use rocksdb_backend::*;

mod db;
use db::*;

mod app_state;
use app_state::*;

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();
    let figment = rocket.figment();

    let config: Config = figment.extract().expect("config");
    let db: Arc<dyn Database> = Arc::new(RocksDB::open(&config.db_path).expect("db open"));
    let state = AppState { config, db };

    rocket
        .manage(state)
        .mount("/", routes![put_require, get_require])
}
