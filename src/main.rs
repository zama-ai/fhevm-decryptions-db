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

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use std::fs::remove_dir_all;

    fn clean_db() {
        match remove_dir_all("/tmp/zbc-oracle-db-testing") {
            Ok(_) => (),
            _ => (),
        }
    }

    #[test]
    fn invalid_get_route() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/xyz").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn invalid_get_key_size() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/require/ab").dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn invalid_get_key_format() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client
            .get("/require/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaXaaaa")
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn unknown_get_key() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client
            .get("/require/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn put_and_get_success() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let json = r##"{
                "value": true,
                "hex_signature": "bbbb"
              }"##;
        let uri = "/require/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

        let response = client
            .put(uri)
            .header(ContentType::JSON)
            .body(json)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let response = client.get(uri).dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}
