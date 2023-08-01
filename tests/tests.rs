// BSD 3-Clause Clear License

// Copyright © 2023 ZAMA.
// All rights reserved.

use fhevm_decryptions_db::routes::Decryption;
use rocket::http::{ContentType, Status};
use serde_json;

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
    let response = client.get("/decryption/ab").dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn get_invalid_key_format() {
    let client = setup();
    let response = client
        .get("/decryption/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaXaaaa")
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn get_unknown_key() {
    let client = setup();
    let response = client
        .get("/decryption/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn put_and_get_success() {
    let client = setup();
    let json = r##"{
                "value": 42,
                "signature": "YmJiYg=="
              }"##;
    let uri = "/decryption/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

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
    let decryption: Result<Decryption, _> = serde_json::from_str(&body);
    assert!(decryption.is_ok());
    let decryption = decryption.unwrap();
    assert_eq!(decryption.value, 42);
    assert_eq!(decryption.signature, "YmJiYg==");
}

#[test]
fn put_updates() {
    let client = setup();
    let json = r##"{
                "value": 42,
                "signature": "YmJiYg=="
              }"##;
    let uri = "/decryption/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

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
    let decryption: Result<Decryption, _> = serde_json::from_str(&body);
    assert!(decryption.is_ok());
    let decryption = decryption.unwrap();
    assert_eq!(decryption.value, 42);
    assert_eq!(decryption.signature, "YmJiYg==");

    let json = r##"{
            "value": 77,
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
    let decryption: Result<Decryption, _> = serde_json::from_str(&body);
    assert!(decryption.is_ok());
    let decryption = decryption.unwrap();
    assert_eq!(decryption.value, 77);
    assert_eq!(decryption.signature, "Yg==");
}

#[test]
fn put_invalid_does_not_update() {
    let client = setup();
    let json = r##"{
                "value": 42,
                "signature": "YmJiYg=="
              }"##;
    let uri = "/decryption/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let response = client
        .put(uri)
        .header(ContentType::JSON)
        .body(json)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    let json = r##"{
            "value" 78,
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
    let decryption: Result<Decryption, _> = serde_json::from_str(&body);
    assert!(decryption.is_ok());
    let decryption = decryption.unwrap();
    assert_eq!(decryption.value, 42);
    assert_eq!(decryption.signature, "YmJiYg==");
}

#[test]
fn put_invalid_json() {
    let client = setup();
    let json = r##"{
                "value" 79
                "signature": "mJiYg=="
              }"##;
    let uri = "/decryption/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

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
                "value": 11,
                "signature": "YmJiY"
              }"##;
    let uri = "/decryption/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    let response = client
        .put(uri)
        .header(ContentType::JSON)
        .body(json)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}
