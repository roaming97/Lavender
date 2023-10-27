use super::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::{alphabet, Engine};
use tower::ServiceExt;

/// Tests against a base64 engine to check if the provided string is valid base64 data.
fn test_base64_str(s: &str) -> bool {
    let engine = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());
    engine.decode(s).is_ok()
}

/// Test a route.
///
/// It returns the body as a `String` and its status as a `StatusCode` for asserting.
async fn test(route: &str, key: Option<&str>) -> (String, StatusCode) {
    let config = LavenderConfig::new();
    let lavender_api_hash = "0c508a046e5d93c3405af45332680a7aa3155f43858d009e106a6a4c67ed85c1".to_owned();
    let state = Arc::<AppState>::new(AppState { config, lavender_api_hash });

    let lavender = lavender(state);

    let response = lavender
        .oneshot(
            Request::builder()
                .uri(route)
                .header("lav-api-key", key.unwrap_or_default())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = hyper::body::to_bytes(response.into_body())
        .await
        .unwrap()
        .to_vec();

    (String::from_utf8(body).unwrap(), status)
}

#[tokio::test]
async fn get_single_file() {
    let (text, status) = test(
        "/file?path=day1_master.png&name_only=false",
        Some("TEST_KEY"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(test_base64_str(&text))
}

#[tokio::test]
async fn get_single_file_name() {
    let (text, status) = test(
        "/file?path=day1_master.png&name_only=true",
        Some("TEST_KEY"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(&text, "day1_master.png")
}

#[tokio::test]
async fn get_file_amount() {
    let (text, status) = test("/amount", Some("TEST_KEY")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(&text.parse::<i32>().is_ok())
}

#[tokio::test]
async fn latest_file_root_path() {
    let (text, status) = test("/latest?master=true", Some("TEST_KEY")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(test_base64_str(text.trim()))
}

#[tokio::test]
async fn latest_file_test_dir() {
    let (text, status) = test("/latest?relpath=/test_dir&master=true", Some("TEST_KEY")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(test_base64_str(text.trim()))
}

#[tokio::test]
async fn multiple_latest_files_root_path() {
    let (text, status) = test("/latest?count=3&master=true", Some("TEST_KEY")).await;
    assert_eq!(status, StatusCode::OK);
    for data in text.split('\n') {
        assert!(test_base64_str(data))
    }
}

#[tokio::test]
async fn multiple_latest_files_test_dir() {
    let (text, status) = test(
        "/latest?count=3&relpath=/test_dir&master=true",
        Some("TEST_KEY"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    for data in text.split('\n') {
        assert!(test_base64_str(data))
    }
}

#[tokio::test]
async fn not_found() {
    let (_, status) = test("/notfound", Some("TEST_KEY")).await;
    assert_eq!(status, StatusCode::NOT_FOUND)
}

#[tokio::test]
async fn unauthorized_no_key() {
    let (_, status) = test("/file?path=day1_master.png&name_only=false", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED)
}

#[tokio::test]
async fn unauthorized_invalid_key() {
    let (_, status) = test(
        "/file?path=day1_master.png&name_only=false",
        Some("this key is invalid"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST)
}