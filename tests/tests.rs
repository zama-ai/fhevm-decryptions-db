use rocket::http::{ContentType, Status};
use serde_json;
use zbc_oracle_db::routes::Require;

mod common;

use common::setup;

#[test]
fn get_invalid_route() {
    let client = setup();
    let response = client.get("/xyz").dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn put_invalid_route() {
    let client = setup();
    let response = client.put("/xyz").dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn get_invalid_key_size() {
    let client = setup();
    let response = client.get("/require/ab").dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn get_invalid_key_format() {
    let client = setup();
    let response = client
        .get("/require/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaXaaaa")
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn get_unknown_key() {
    let client = setup();
    let response = client
        .get("/require/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn put_and_get_success() {
    let client = setup();
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
    let client = setup();
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
    let client = setup();
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
    let client = setup();
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
    let client = setup();
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
