use std::{fs::remove_dir_all, path::Path};

use rocket::local::blocking::Client;
use zbc_oracle_db::{build_rocket, config::Config, configure_rocket};

fn clean_db(path: &str) {
    match remove_dir_all(path) {
        _ => (),
    }
    assert!(!Path::exists(Path::new(path)));
}

pub fn setup() -> Client {
    let rocket = build_rocket();
    let config: Config = rocket.figment().extract().expect("config");
    clean_db(&config.db_path);
    Client::tracked(configure_rocket(rocket)).expect("valid rocket instance")
}
