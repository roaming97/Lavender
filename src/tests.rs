use std::env;

use crate::file::LavenderEntry;
use crate::routes::*;

use super::*;
use axum_test::http::{HeaderName, HeaderValue};
use axum_test::{TestRequest, TestServer};
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::{alphabet, Engine};
use serde_json::json;

const TEST_API_KEY: Option<&str> = Some("TEST_KEY");

/// Tests against a base64 engine to check if the provided string is valid base64 data.
fn test_base64_str(s: &str) -> bool {
    if s.is_empty() {
        eprintln!("Empty Base64 string!");
        return false;
    }
    let engine = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());
    engine.decode(s).is_ok()
}

/// Test a route.
///
/// # Returns
/// `TestRequest` so it can be awaited later.
/// This ensures a safer way to send it between threads.
fn test<Q: serde::Serialize>(route: &str, query: Q, key: Option<&str>) -> TestRequest {
    let config = Config::new();
    let state = Arc::<Config>::new(config);

    let lavender = lavender(state);
    let server = TestServer::new(lavender).unwrap();

    env::set_var(
        "LAVENDER_API_HASH",
        "0c508a046e5d93c3405af45332680a7aa3155f43858d009e106a6a4c67ed85c1",
    );

    server.get(route).add_query_params(query).add_header(
        HeaderName::from_static("lav-api-key"),
        HeaderValue::from_str(key.unwrap_or_default()).unwrap(),
    )
}

// * ROUTE TESTS

// /file

#[tokio::test]
async fn get_single_file() {
    let query = GetFileParams {
        path: "./artwork/everydays/thumbnails/day1.webp".into(),
    };
    let response = test("/file", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let file = response.json::<LavenderEntry>();
    assert!(test_base64_str(&file.base64()));
}

// /amount

#[tokio::test]
async fn get_file_amount() {
    let response = test("/amount", json!({}), TEST_API_KEY).await;
    response.assert_status_ok();
    assert!(&response.text().parse::<i32>().is_ok());
}

// /latest

#[tokio::test]
async fn latest_file_root_path() {
    let query = LatestFilesParams {
        kind: Some(ReturnKind::Entries),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let json = response.json::<LatestFilesResponse>();
    assert!(json.entries.is_some());
    assert!(test_base64_str(&json.entries.unwrap()[0].base64()));
}

#[tokio::test]
async fn multiple_latest_files_root_path() {
    let query = LatestFilesParams {
        relpath: Some("/artwork".into()),
        count: Some(3),
        kind: Some(ReturnKind::Both),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let json = response.json::<LatestFilesResponse>();
    assert!(json.entries.is_some());
    for entry in json.entries.unwrap() {
        assert!(test_base64_str(&entry.base64()));
    }
}

#[tokio::test]
async fn latest_image_root_path() {
    let query = LatestFilesParams {
        relpath: Some("/artwork".into()),
        kind: Some(ReturnKind::Both),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let json = response.json::<LatestFilesResponse>();
    assert!(json.entries.is_some());
    assert!(test_base64_str(&json.entries.unwrap()[0].base64()));
}

#[tokio::test]
async fn latest_thumbnail_root_path() {
    let query = LatestFilesParams {
        relpath: Some("/artwork".into()),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let json = response.json::<LatestFilesResponse>();
    assert!(json.thumbnails.is_some());
    for entry in json.thumbnails.unwrap() {
        assert!(test_base64_str(&entry.base64()));
    }
}

#[tokio::test]
async fn latest_thumbnails_root_path() {
    let query = LatestFilesParams {
        relpath: Some("/artwork".into()),
        count: Some(4),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let json = response.json::<LatestFilesResponse>();
    assert!(json.thumbnails.is_some());
    for entry in json.thumbnails.unwrap() {
        assert!(test_base64_str(&entry.base64()));
    }
}

#[tokio::test]
async fn latest_thumbnails_root_path_with_offset() {
    let query = LatestFilesParams {
        relpath: Some("/artwork".into()),
        count: Some(4),
        offset: Some(2),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let json = response.json::<LatestFilesResponse>();
    assert!(json.thumbnails.is_some());
    for entry in json.thumbnails.unwrap() {
        assert!(test_base64_str(&entry.base64()));
    }
}

#[tokio::test]
async fn latest_video_root_path() {
    let query = LatestFilesParams {
        relpath: Some("/video".into()),
        kind: Some(ReturnKind::Both),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let json = response.json::<LatestFilesResponse>();
    assert!(json.entries.is_some());
    assert!(test_base64_str(&json.entries.unwrap()[0].base64()));
}

#[tokio::test]
async fn latest_video_thumbnail_root_path() {
    let query = LatestFilesParams {
        relpath: Some("/video".into()),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let json = response.json::<LatestFilesResponse>();
    assert!(json.thumbnails.is_some());
    assert!(test_base64_str(&json.thumbnails.unwrap()[0].base64()));
}

#[tokio::test]
async fn latest_file_test_dir() {
    let query = LatestFilesParams {
        relpath: Some("/test_dir".into()),
        kind: Some(ReturnKind::Entries),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let json = response.json::<LatestFilesResponse>();
    assert!(json.entries.is_some());
    assert!(test_base64_str(&json.entries.unwrap()[0].base64()));
}

#[tokio::test]
async fn multiple_latest_files_test_dir() {
    let query = LatestFilesParams {
        count: Some(4),
        relpath: Some("/test_dir".into()),
        kind: Some(ReturnKind::Entries),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let json = response.json::<LatestFilesResponse>();
    assert!(json.entries.is_some());
    for entry in json.entries.unwrap() {
        assert!(test_base64_str(&entry.base64()));
    }
}

// ! ERROR TESTS

#[tokio::test]
async fn not_found() {
    let response = test("/notfound", json!({}), TEST_API_KEY).await;
    response.assert_status_not_found();
}

#[tokio::test]
async fn get_nonexistent_file() {
    let query = GetFileParams {
        path: "i.dont.exist".into(),
    };
    let response = test("/file", query, TEST_API_KEY).await;
    response.assert_status_bad_request();
}

#[tokio::test]
async fn unauthorized_no_key() {
    let query = GetFileParams {
        path: "day1_master.png".into(),
    };
    let response = test("/file", query, None).await;
    response.assert_status_unauthorized();
}

#[tokio::test]
async fn unauthorized_empty_key() {
    let query = GetFileParams {
        path: "day1_master.png".into(),
    };
    let response = test("/file", query, Some("")).await;
    response.assert_status_unauthorized();
}

#[tokio::test]
async fn unauthorized_invalid_key() {
    let query = GetFileParams {
        path: "day1_master.png".into(),
    };
    let response = test("/file", query, Some("this key is invalid")).await;
    response.assert_status_bad_request();
}

#[tokio::test]
async fn latest_zero_files() {
    let query = LatestFilesParams {
        // should default to 1
        count: Some(0),
        kind: Some(ReturnKind::Entries),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_ok();
    let json = response.json::<LatestFilesResponse>();
    assert!(json.entries.is_some());
    assert!(test_base64_str(&json.entries.unwrap()[0].base64()));
}

#[tokio::test]
async fn latest_files_invalid_offset() {
    let query = LatestFilesParams {
        offset: Some(1000),
        ..Default::default()
    };
    let response = test("/latest", query, TEST_API_KEY).await;
    response.assert_status_bad_request();
}
