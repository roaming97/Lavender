use std::fs;
use std::path;

use crate::file;
use image::{imageops, GenericImageView, ImageFormat};
use rocket::http::Status;
use walkdir::WalkDir;

#[get("/file?<path>&<name_only>")]
pub async fn get_file(path: &str, name_only: bool) -> Result<String, Status> {
    if name_only {
        return Ok(path.split('/').last().unwrap().to_owned());
    }

    let filepath = format!("{}/{}", &file::get_media_path(), path);
    let dir = path::Path::new(&filepath);
    match file::LavenderFile::new(dir) {
        Ok(f) => Ok(f.read_base64()),
        Err(_) => Err(Status::BadRequest),
    }
}

#[get("/amount")]
pub async fn file_amount() -> String {
    let v: Vec<walkdir::DirEntry> = WalkDir::new(file::get_media_path())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.file_name()
                    .to_str()
                    .map(|s| !s.starts_with('.') && !s.contains(file::MASTER_FILE_SUFFIX))
                    .unwrap_or(false)
        })
        .collect();

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
                    if datatype.is_image() && master {
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
        if let Ok(f) = file::LavenderFile::new(path.as_path()) {
            output.push_str(&format!("{}\n", f.read_base64()));
        } else {
            return Err(Status::BadRequest);
        }
    }

    Ok(output)
}

#[get("/optimize")]
pub fn create_optimized_images(_key: crate::api::ApiKey) -> &'static str {
    let path = file::get_media_path();
    let dir = path::Path::new(&path);
    for entry in fs::read_dir(dir).expect("Could not read path to optimize images!") {
        let entry = entry.unwrap();
        let path = entry.path();

        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap();
            if file::DataType::from_extension(ext).is_image() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let master_filename =
                    format!("{}_master.{}", file_name.split('.').next().unwrap(), ext);
                let target_path = dir.join(master_filename);

                if target_path.exists() || file_name.contains("_master.png") {
                    continue;
                }

                match image::open(&path) {
                    Ok(i) => {
                        let (width, height) = i.dimensions();
                        if width > 640 || height > 640 {
                            let nwidth = (width as f32 * 0.25) as u32;
                            let nheight = (height as f32 * 0.25) as u32;
                            println!("Loaded image with size: {}x{}", nwidth, nheight);
                            let img = i.resize(nwidth, nheight, imageops::FilterType::CatmullRom);
                            let new_filename = format!(
                                "{}_master.png",
                                path.file_stem().unwrap().to_string_lossy()
                            );
                            let new_path = path::Path::new(dir).join(new_filename);
                            img.save_with_format(new_path, ImageFormat::Png).unwrap();
                        }
                    }
                    Err(e) => println!("File \'{:?}\' not found!: {:?}", entry.path(), e),
                }
            }
        }
    }

    "Done"
}
