use std::path::Path;
use std::sync::Arc;

use crate::api::Key;
use crate::{file, ShuttleState};
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
    State(data): State<Arc<ShuttleState>>,
    Query(query): Query<GetFileParams>,
    Key(_): Key,
) -> Result<Json<file::LavenderEntry>, StatusCode> {
    let path = Path::new(&query.path);
    let filepath = format!("{}/{}", &data.config.media_path, path.display());
    file::LavenderEntry::new(filepath).map_or(Err(StatusCode::NOT_FOUND), |file| {
        if file.is_valid() {
            Ok(Json(file))
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    })
}

pub async fn file_amount(State(data): State<Arc<ShuttleState>>, Key(_): Key) -> String {
    file::scan_fs(&data.config.media_path, true)
        .into_iter()
        .filter(|f| f.path().extension().unwrap_or_default().ne("webp"))
        .count()
        .to_string()
}

#[derive(Serialize, Deserialize, Default)]
pub struct LatestFilesParams {
    pub count: Option<usize>,
    pub relpath: Option<String>,
    pub offset: Option<usize>,
    pub thumbnail: bool,
}

pub async fn get_latest_files(
    State(data): State<Arc<ShuttleState>>,
    Query(query): Query<LatestFilesParams>,
    Key(_): Key,
) -> Result<Json<Vec<file::LavenderEntry>>, StatusCode> {
    let count = query.count.unwrap_or(1);
    let path = format!(
        "{}{}{}",
        &data.config.media_path,
        std::path::MAIN_SEPARATOR,
        &query.relpath.unwrap_or_default()
    );
    let mut walk: Vec<walkdir::DirEntry> = file::scan_fs(path, true)
        .into_iter()
        .filter(|f| {
            if query.thumbnail {
                f.path().extension().unwrap_or_default().eq("webp")
            } else {
                true
            }
        })
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

    let mut output = Vec::<file::LavenderEntry>::new();

    for entry in walk {
        // println!("{}", entry.path().display());
        if let Ok(f) = file::LavenderEntry::new(entry.path()) {
            if !f.is_valid() {
                return Err(StatusCode::BAD_REQUEST);
            }
            output.push(f);
        }
    }

    Ok(Json(output))
}
