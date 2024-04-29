use std::path::Path;
use std::sync::Arc;

use crate::api::Key;
use crate::{file, Config};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Result;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetFileParams {
    pub path: String,
}

pub async fn get_file(
    State(data): State<Arc<Config>>,
    Query(query): Query<GetFileParams>,
    Key(_): Key,
) -> Result<Json<file::LavenderEntry>, StatusCode> {
    let path = Path::new(&query.path);
    let filepath = format!("{}/{}", &data.media_path, path.display());
    file::LavenderEntry::new(filepath).map_or(Err(StatusCode::BAD_REQUEST), |file| Ok(Json(file)))
}

pub async fn file_amount(State(data): State<Arc<Config>>, Key(_): Key) -> String {
    file::scan_fs(&data.media_path, true)
        .into_iter()
        .filter(|f| f.path().extension().unwrap_or_default().ne("webp"))
        .count()
        .to_string()
}

#[derive(Serialize, Deserialize)]
pub enum ReturnKind {
    Entries,
    Thumbnails,
    Both,
}

impl Default for ReturnKind {
    fn default() -> Self {
        Self::Both
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct LatestFilesParams {
    pub count: Option<usize>,
    pub relpath: Option<String>,
    pub offset: Option<usize>,
    pub kind: ReturnKind,
}

#[derive(Serialize, Deserialize, Default)]
pub struct LatestFilesResponse {
    pub entries: Option<Vec<file::LavenderEntry>>,
    pub thumbnails: Option<Vec<file::LavenderEntry>>,
}

fn latest_entries(walk: &[walkdir::DirEntry]) -> Vec<file::LavenderEntry> {
    walk.iter()
        .map(|e| {
            let path = e.path();
            match file::LavenderEntry::new(path) {
                Ok(f) => f,
                Err(e) => panic!("Panicked while creating Lavender entry: {e:?}"),
            }
        })
        .collect()
}

fn latest_thumbnails(walk: &[walkdir::DirEntry]) -> Vec<file::LavenderEntry> {
    walk.iter()
        .map(|e| {
            let path = e.path();
            let thumbnail_path = format!(
                "{0}{1}thumbnails{1}{2}.webp",
                path.parent().unwrap().display(),
                std::path::MAIN_SEPARATOR,
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            match file::LavenderEntry::new(thumbnail_path) {
                Ok(f) => f,
                Err(e) => panic!("Panicked while creating Lavender entry: {e:?}"),
            }
        })
        .collect()
}

pub async fn get_latest_files(
    State(data): State<Arc<Config>>,
    Query(query): Query<LatestFilesParams>,
    Key(_): Key,
) -> Result<Json<LatestFilesResponse>, StatusCode> {
    let count = query.count.unwrap_or(1);
    let path = format!(
        "{}{}{}",
        &data.media_path,
        std::path::MAIN_SEPARATOR,
        &query.relpath.unwrap_or_default()
    );
    let mut walk: Vec<walkdir::DirEntry> = file::scan_fs(path, true)
        .into_iter()
        .filter(|f| f.path().extension().unwrap_or_default().ne("webp"))
        .collect();

    if walk.is_empty() || query.offset.unwrap_or_default() >= walk.len() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let count = count.clamp(1, walk.len());
    if let Some(offset) = query.offset {
        walk = walk.drain(offset..count + offset).collect();
    } else {
        walk.truncate(count);
    }

    let mut response = LatestFilesResponse::default();

    match query.kind {
        ReturnKind::Entries => response.entries = Some(latest_entries(&walk)),
        ReturnKind::Thumbnails => response.thumbnails = Some(latest_thumbnails(&walk)),
        ReturnKind::Both => {
            response.entries = Some(latest_entries(&walk));
            response.thumbnails = Some(latest_thumbnails(&walk));
        }
    }

    Ok(Json(response))
}
