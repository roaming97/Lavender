use std::path::{self, Path};
use std::sync::Arc;

use crate::api::ApiKey;
use crate::file;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Result;
use axum::Json;
use image::{imageops, GenericImageView, ImageFormat};
use serde::{Serialize, Deserialize};

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
        },
        Err(_) => Err(StatusCode::NOT_FOUND)
    }

}

pub async fn file_amount(
    State(data): State<Arc<file::LavenderConfig>>,
    ApiKey(_): ApiKey,
) -> String {
    file::scan_fs(&data.server.media_path, true, false)
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

    let mut walk: Vec<walkdir::DirEntry> = file::scan_fs(path, false, query.thumbnail)
        .into_iter()
        .filter(|e| {
            let extension = e.path().extension().unwrap_or_default();
            let datatype = file::DataType::from(extension, Some(&data));
            if datatype.is_type(&file::DataType::Image) && query.thumbnail {
                if type_filter.is_type(&file::DataType::Image) {
                    e.file_name()
                        .to_string_lossy()
                        .contains(file::MASTER_IMAGE_SUFFIX)
                } else {
                    e.file_name()
                        .to_string_lossy()
                        .contains(file::VIDEO_THUMBNAIL_SUFFIX)
                }
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

pub async fn create_optimized_images(
    State(data): State<Arc<file::LavenderConfig>>,
    ApiKey(_): ApiKey,
) -> StatusCode {
    let v = file::scan_fs(&data.server.media_path, true, false);
    for entry in v {
        let path = entry.path();
        let parent = path.parent().unwrap().to_str().unwrap();
        let filename = path.file_stem().unwrap().to_str().unwrap();
        let extension = path.extension().unwrap().to_str().unwrap();
        if !file::DataType::from(extension.to_ascii_lowercase().as_str(), Some(&data))
            .is_type(&file::DataType::Image)
        {
            continue;
        }
        let target = format!(
            "{}{}{}{}{}",
            parent,
            path::MAIN_SEPARATOR,
            filename,
            file::MASTER_IMAGE_SUFFIX,
            extension
        );
        if path::Path::new(&target).exists() {
            continue;
        } else {
            match image::open(path) {
                Ok(i) => {
                    let (w, h) = i.dimensions();
                    if w > 640 || h > 640 {
                        let nwidth = (w as f32 * 0.25) as u32;
                        let nheight = (h as f32 * 0.25) as u32;
                        let img = i.resize(nwidth, nheight, imageops::FilterType::CatmullRom);
                        img.save_with_format(target, ImageFormat::Png).unwrap();
                    }
                }
                Err(e) => {
                    println!(
                        "Failed to open \'{}\': {}",
                        entry.path().to_str().unwrap(),
                        e
                    );
                    return StatusCode::INTERNAL_SERVER_ERROR;
                }
            }
        }
    }
    StatusCode::OK
}
