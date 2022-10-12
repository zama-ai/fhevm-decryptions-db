#[macro_use]
extern crate rocket;

mod config;
mod rocksdb_store;
mod routes;

use config::*;
use rocksdb_store::*;
use routes::*;
use std::sync::Arc;

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();
    let figment = rocket.figment();

    let config: Config = figment.extract().expect("config");
    let db = Arc::new(RocksDBStore::open(&config.db_path).expect("db open"));

    rocket
        .manage(db)
        .mount("/", routes![put_require, get_require])
}

#[cfg(test)]
mod test {
    use crate::routes::Require;

    use super::rocket;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use serde_json;
    use std::fs::remove_dir_all;

    fn clean_db() {
        match remove_dir_all("/tmp/zbc-oracle-db-testing") {
            Ok(_) => (),
            _ => (),
        }
    }

    #[test]
    fn get_invalid_route() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/xyz").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn get_invalid_key_size() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/require/ab").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn get_invalid_key_format() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client
            .get("/require/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaXaaaa")
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn get_unknown_key() {
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
                "signature": "YmJiYg=="
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
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let body = response.into_string();
        assert!(body.is_some());
        let body = body.unwrap();
        let require: Result<Require, _> = serde_json::from_str(&body);
        assert!(require.is_ok());
        let require = require.unwrap();
        assert!(require.value);
        assert_eq!(require.signature, "YmJiYg==");
    }

    #[test]
    fn put_updates() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let json = r##"{
                "value": true,
                "signature": "YmJiYg=="
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
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let body = response.into_string();
        assert!(body.is_some());
        let body = body.unwrap();
        let require: Result<Require, _> = serde_json::from_str(&body);
        assert!(require.is_ok());
        let require = require.unwrap();
        assert!(require.value);
        assert_eq!(require.signature, "YmJiYg==");

        let json = r##"{
            "value": false,
            "signature": "Yg=="
          }"##;
        let response = client
            .put(uri)
            .header(ContentType::JSON)
            .body(json)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let response = client.get(uri).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let body = response.into_string();
        assert!(body.is_some());
        let body = body.unwrap();
        let require: Result<Require, _> = serde_json::from_str(&body);
        assert!(require.is_ok());
        let require = require.unwrap();
        assert!(!require.value);
        assert_eq!(require.signature, "Yg==");
    }

    #[test]
    fn put_invalid_does_not_update() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let json = r##"{
                "value": true,
                "signature": "YmJiYg=="
              }"##;
        let uri = "/require/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

        let response = client
            .put(uri)
            .header(ContentType::JSON)
            .body(json)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let json = r##"{
            "value" false,
            "signature" "Yg=="
          }"##;
        let response = client
            .put(uri)
            .header(ContentType::JSON)
            .body(json)
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);

        let response = client.get(uri).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let body = response.into_string();
        assert!(body.is_some());
        let body = body.unwrap();
        let require: Result<Require, _> = serde_json::from_str(&body);
        assert!(require.is_ok());
        let require = require.unwrap();
        assert!(require.value);
        assert_eq!(require.signature, "YmJiYg==");
    }

    #[test]
    fn put_invalid_json() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let json = r##"{
                "value" true
                "signature": "mJiYg=="
              }"##;
        let uri = "/require/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

        let response = client
            .put(uri)
            .header(ContentType::JSON)
            .body(json)
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn put_invalid_signature() {
        clean_db();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let json = r##"{
                "value": true,
                "signature": "YmJiY"
              }"##;
        let uri = "/require/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

        let response = client
            .put(uri)
            .header(ContentType::JSON)
            .body(json)
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }
}
