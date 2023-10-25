use std::fs;
use std::path;
use std::path::MAIN_SEPARATOR;

use crate::file;
use crate::file::MASTER_FILE_SUFFIX;
use image::{imageops, GenericImageView, ImageFormat};
use rocket::http::Status;

#[get("/file?<path>&<name_only>")]
pub async fn get_file(path: &str, name_only: bool) -> Result<String, Status> {
    if name_only {
        return Ok(path.split('/').last().unwrap().to_owned());
    }
    let filepath = format!("{}/{}", &file::get_media_path(), path);
    let file = file::LavenderFile::new(filepath);
    if file.is_valid() {
        Ok(file.read_base64())
    } else {
        Err(Status::BadRequest)
    }
}

#[get("/amount")]
pub async fn file_amount() -> String {
    let v = file::get_all_files_recursively();
    v.len().to_string()
}

#[get("/latest?<count>&<relpath>&<master>")]
pub async fn get_latest_files(
    count: Option<usize>,
    relpath: Option<&str>,
    master: bool,
) -> Result<String, Status> {
    let path = format!(
        "{}{}{}",
        file::get_media_path(),
        path::MAIN_SEPARATOR,
        relpath.unwrap_or_default()
    );
    let mut entries: Vec<_> = match fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|e| {
                let entry = e.ok()?;
                let mut path = entry.path();
                let extension = path.extension()?.to_str()?;
                let datatype = file::DataType::from_extension(extension);
                let metadata = entry.metadata().ok()?;
                let modified = metadata.modified().ok()?;
                /*
                One cannot rely on master images directly since they can be created
                anytime differing with the original files' dates, so let's filter those
                out and add the master suffix later.
                This helps the later date sorting to be more accurate.
                */
                if metadata.is_file() && !path.to_string_lossy().contains(file::MASTER_FILE_SUFFIX)
                {
                    if datatype.is_type(file::DataType::Image) && master {
                        path = path
                            .to_string_lossy()
                            .replace(
                                &format!(".{}", extension),
                                &format!("{}{}", file::MASTER_FILE_SUFFIX, extension),
                            )
                            .into();
                    }
                    Some((path, modified))
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => return Err(Status::InternalServerError),
    };

    entries.sort_by(|(_, t1), (_, t2)| t2.cmp(t1));
    let count = count.unwrap_or(1).min(entries.len());
    entries.truncate(count);

    let mut output = String::new();

    for (path, _) in entries {
        println!("{}", path.to_str().unwrap());
        let f = file::LavenderFile::new(path);
        if f.is_valid() {
            output.push_str(&format!("{}\n", f.read_base64()));
        } else {
            return Err(Status::BadRequest);
        }
    }

    Ok(output)
}

#[get("/optimize")]
pub fn create_optimized_images(_key: crate::api::ApiKey) -> Status {
    let v = file::get_all_files_recursively();
    for entry in v {
        let path = entry.path();
        let parent = path.parent().unwrap().to_str().unwrap();
        let filename = path.file_stem().unwrap().to_str().unwrap();
        let extension = path.extension().unwrap().to_str().unwrap();
        if !file::DataType::from_extension(extension.to_ascii_lowercase().as_str())
            .is_type(file::DataType::Image)
        {
            continue;
        }
        let target = format!(
            "{}{}{}{}{}",
            parent, MAIN_SEPARATOR, filename, MASTER_FILE_SUFFIX, extension
        );
        dbg!(&target);
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
                    return Status::InternalServerError;
                }
            }
        }
    }
    Status::Ok
}