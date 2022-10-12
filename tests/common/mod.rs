use std::fs::remove_dir_all;

use rocket::local::blocking::Client;
use zbc_oracle_db::build_rocket;

fn clean_db() {
    match remove_dir_all("/tmp/zbc-oracle-db-testing") {
        _ => (),
    }
}

pub fn setup() -> Client {
    clean_db();
    Client::tracked(build_rocket()).expect("valid rocket instance")
}
