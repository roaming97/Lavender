use std::path::Path;

use crate::file;
use crate::{api::ApiKey, file::LavenderFile};
use image::{imageops, GenericImageView, ImageFormat};
use rocket::http::Status;

#[get("/file?<path>&<name_only>")]
pub async fn get_file(path: &str, name_only: bool) -> Result<String, Status> {
    if name_only {
        return Ok(path.split('/').last().unwrap().to_owned());
    }

    let filepath = format!("{}/{}", &file::get_media_path(), path);
    let dir = Path::new(&filepath);
    match file::LavenderFile::new(dir) {
        Ok(f) => Ok(f.read_base64()),
        Err(_) => Err(Status::BadRequest),
    }
}

#[get("/amount")]
pub async fn file_amount() -> Result<String, Status> {
    match std::fs::read_dir(file::get_media_path()) {
        Ok(dir) => {
            let v: Vec<std::fs::DirEntry> = dir
                .filter_map(|e| {
                    let entry = e.ok()?;
                    let file = LavenderFile::new(entry.path().as_path()).unwrap();
                    if file.datatype.is_image()
                        && entry
                            .path()
                            .to_string_lossy()
                            .contains(file::MASTER_FILE_SUFFIX)
                    {
                        None
                    } else {
                        Some(entry) 
                    }
                })
                .collect();
            Ok(v.len().to_string())
        }
        Err(_) => Err(Status::BadRequest),
    }
}

// TODO: There must be a way I can manually input metadata for the files.
// Once that is done I could separate them by categories and write their original
// creation dates, skipping in the process the master suffix step.
#[get("/latest?<count>&<master>")]
pub async fn get_latest_files(count: Option<usize>, master: bool) -> Result<String, Status> {
    let media_path = file::get_media_path();
    let mut entries: Vec<_> = match std::fs::read_dir(&media_path) {
        Ok(entries) => entries
            .filter_map(|e| {
                let entry = e.ok()?;
                let mut path = entry.path();
                let datatype = file::DataType::from_extension(path.extension()?.to_str()?);
                let metadata = entry.metadata().ok()?;
                let modified = metadata.modified().ok()?;
                /*
                One cannot rely on master images directly, since they can be created
                way after the original versions were, so let's filter those out and
                add the master suffix later. This returns more accurate file sorting.
                */
                if metadata.is_file() && !path.to_string_lossy().contains(file::MASTER_FILE_SUFFIX)
                {
                    if datatype.is_image() && master {
                        path = path
                            .to_string_lossy()
                            .replace(".png", &format!("{}.png", file::MASTER_FILE_SUFFIX))
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
        if let Ok(f) = file::LavenderFile::new(path.as_path()) {
            output.push_str(&format!("{}\n", f.read_base64()));
        } else {
            return Err(Status::BadRequest);
        }
    }

    Ok(output)
}

#[get("/optimize")]
pub fn create_optimized_images(_key: ApiKey) -> &'static str {
    let path = file::get_media_path();
    let dir = Path::new(&path);
    for entry in std::fs::read_dir(dir).expect("Could not read path to optimize images!") {
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
                            let new_path = std::path::Path::new(dir).join(new_filename);
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
