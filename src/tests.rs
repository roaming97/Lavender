use super::rocket;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::{alphabet, Engine};
use rocket::http::Status;
use rocket::local::blocking::Client;

/// Tests against a base64 engine to check if the provided string is valid base64 data.
fn test_base64_str(s: &str) -> bool {
    let engine = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());
    engine.decode(s).is_ok()
}

#[test]
fn get_single_file() {
    let client = Client::untracked(rocket()).expect("Valid rocket.rs instance");
    let response = client
        .get(uri!(super::get_file("day1_master.png", false)))
        .dispatch();

    assert_eq!(response.status().clone(), Status::Ok);
    assert!(test_base64_str(&response.into_string().unwrap()));
}

#[test]
fn get_single_file_name() {
    let client = Client::untracked(rocket()).expect("Valid rocket.rs instance");
    let response = client
        .get(uri!(super::get_file("day1_master.png", true)))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(response.into_string().is_some());
}

#[test]
fn get_file_amount() {
    let client = Client::untracked(rocket()).expect("Valid rocket.rs instance");
    let response = client.get(uri!(super::file_amount)).dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(response.into_string().unwrap().parse::<i32>().is_ok());
}

#[test]
fn latest_file_root_path() {
    let client = Client::untracked(rocket()).expect("Valid rocket.rs instance");
    let response = client
        .get(uri!(super::get_latest_files(
            Some(1),
            Option::<&str>::None,
            true
        )))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(test_base64_str(
        response.into_string().unwrap().as_str().trim()
    ));
}

#[test]
fn latest_file_test_dir() {
    let client = Client::untracked(rocket()).expect("Valid rocket.rs instance");
    let response = client
        .get(uri!(super::get_latest_files(
            Some(1),
            Some("/test_dir"),
            true
        )))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert!(test_base64_str(
        response.into_string().unwrap().as_str().trim()
    ));
}

#[test]
fn multiple_latest_files_root_path() {
    let client = Client::untracked(rocket()).expect("Valid rocket.rs instance");
    let response = client
        .get(uri!(super::get_latest_files(
            Some(3),
            Option::<&str>::None,
            true
        )))
        .dispatch();

    assert_eq!(response.status().clone(), Status::Ok);

    for data in response.into_string().unwrap().as_str().split('\n') {
        assert!(test_base64_str(data));
    }
}

#[test]
fn multiple_latest_files_test_dir() {
    let client = Client::untracked(rocket()).expect("Valid rocket.rs instance");
    let response = client
        .get(uri!(super::get_latest_files(
            Some(3),
            Some("/test_dir"),
            true
        )))
        .dispatch();

    assert_eq!(response.status().clone(), Status::Ok);

    for data in response.into_string().unwrap().as_str().split('\n') {
        assert!(test_base64_str(data));
    }
}

#[test]
fn optimize_images() {
    let client = Client::untracked(rocket()).expect("Valid rocket.rs instance");
    let response = client.get(uri!(super::create_optimized_images)).dispatch();
    assert_eq!(response.status(), Status::Ok);
}
