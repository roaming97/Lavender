use std::path;
use std::sync::Arc;

use crate::api::ApiKey;
use crate::file;
use crate::file::LavenderConfig;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Result;
use image::{imageops, GenericImageView, ImageFormat};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetFileParams {
    path: String,
    name_only: bool,
}

pub async fn get_file(
    State(data): State<Arc<LavenderConfig>>,
    Query(query): Query<GetFileParams>,
    ApiKey(_): ApiKey,
) -> Result<String, StatusCode> {
    let path = query.path;
    let name_only = query.name_only;
    let media_path = &data.server.media_path;
    let filepath = format!("{}/{}", media_path, path);
    let file = file::LavenderFile::new(filepath);

    if file.is_valid() {
        if name_only {
            return Ok(path.split('/').last().unwrap().to_owned());
        }
        Ok(file.read_base64())
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn file_amount(State(data): State<Arc<LavenderConfig>>, ApiKey(_): ApiKey) -> String {
    file::get_all_files_recursively(&data.server.media_path)
        .len()
        .to_string()
}

#[derive(Deserialize)]
pub struct LatestFilesParams {
    count: Option<usize>,
    relpath: Option<String>,
    filetype: Option<String>,
    master: bool,
}

pub async fn get_latest_files(
    State(data): State<Arc<LavenderConfig>>,
    Query(query): Query<LatestFilesParams>,
    ApiKey(_): ApiKey,
) -> Result<String, StatusCode> {
    let count = query.count.unwrap_or(1);
    let media_path = &data.server.media_path;
    let relpath = query.relpath.unwrap_or_default();
    let path = format!("{}{}{}", media_path, path::MAIN_SEPARATOR, &relpath);
    let type_filter = file::DataType::from(query.filetype.unwrap_or_default(), Some(&data));

    let mut walk: Vec<walkdir::DirEntry> = file::get_all_files_recursively(path)
        .into_iter()
        .filter(|e| {
            let extension = e.path().extension().unwrap_or_default();
            let datatype = file::DataType::from(extension, Some(&data));
            if datatype.is_type(&file::DataType::Image)
                && e.path().to_string_lossy().contains(&format!(
                    "{}{}",
                    file::MASTER_FILE_SUFFIX,
                    extension.to_string_lossy()
                ))
            {
                query.master
            } else if !type_filter.is_type(&file::DataType::Unknown) {
                datatype.is_type(&type_filter)
            } else {
                true
            }
        })
        .collect();

    if walk.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let count = count.clamp(1, walk.len());
    walk.truncate(count);

    let mut output = String::new();

    for entry in walk {
        println!("{}", &entry.path().to_string_lossy());
        let f = file::LavenderFile::new(entry.path());
        if !f.is_valid() {
            return Err(StatusCode::BAD_REQUEST);
        }
        output.push_str(&format!("{}\n", f.read_base64()));
    }

    let output = output.trim_end().to_owned();
    Ok(output)
}

pub async fn create_optimized_images(
    State(data): State<Arc<LavenderConfig>>,
    ApiKey(_): ApiKey,
) -> StatusCode {
    let v = file::get_all_files_recursively(&data.server.media_path);
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
            file::MASTER_FILE_SUFFIX,
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
