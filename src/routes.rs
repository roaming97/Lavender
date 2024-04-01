use std::path::Path;
use std::sync::Arc;

use crate::api::ApiKey;
use crate::file;
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
    State(data): State<Arc<file::LavenderConfig>>,
    Query(query): Query<GetFileParams>,
    ApiKey(_): ApiKey,
) -> Result<Json<file::LavenderFile>, StatusCode> {
    let path = Path::new(&query.path);
    let filepath = format!("{}/{}", &data.server.media_path, path.display());
    match file::LavenderFile::new(filepath) {
        Ok(file) => {
            if file.is_valid() {
                Ok(Json(file))
            } else {
                Err(StatusCode::BAD_REQUEST)
            }
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn file_amount(
    State(data): State<Arc<file::LavenderConfig>>,
    ApiKey(_): ApiKey,
) -> String {
    file::scan_fs(&data.server.media_path, true)
        .len()
        .to_string()
}

#[derive(Serialize, Deserialize, Default)]
pub struct LatestFilesParams {
    pub count: Option<usize>,
    pub relpath: Option<String>,
    pub filetype: Option<String>,
    pub offset: Option<usize>,
    pub thumbnail: bool,
}

pub async fn get_latest_files(
    State(data): State<Arc<file::LavenderConfig>>,
    Query(query): Query<LatestFilesParams>,
    ApiKey(_): ApiKey,
) -> Result<Json<Vec<file::LavenderFile>>, StatusCode> {
    let count = query.count.unwrap_or(1);
    let path = format!(
        "{}{}{}",
        &data.server.media_path,
        std::path::MAIN_SEPARATOR,
        &query.relpath.unwrap_or_default()
    );
    let type_filter = file::DataType::from(query.filetype.unwrap_or_default(), Some(&data));

    let mut walk: Vec<walkdir::DirEntry> = file::scan_fs(path, true)
        .into_iter()
        .filter(|e| {
            let extension = e.path().extension().unwrap_or_default();
            let datatype = file::DataType::from(extension, Some(&data));
            if !datatype.is_type(&file::DataType::Audio) && query.thumbnail {
                if let Some(dirname) = e.path().parent() {
                    return dirname.to_string_lossy().into_owned().contains("thumbnails");
                } 
                false
            } else if !type_filter.is_type(&file::DataType::Unknown) {
                datatype.is_type(&type_filter)
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

    let mut output = Vec::<file::LavenderFile>::new();

    for entry in walk {
        // println!("{}", entry.path().display());
        if let Ok(f) = file::LavenderFile::new(entry.path()) {
            if !f.is_valid() {
                return Err(StatusCode::BAD_REQUEST);
            }
            output.push(f);
        }
    }

    Ok(Json(output))
}
