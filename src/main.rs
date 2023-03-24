mod api;
mod catchers;
mod file;

use std::{fs, path::Path};

use api::ApiKey;
use catchers::*;
use file::LavenderFile;
use image::{imageops, GenericImageView, ImageFormat};
use rocket::http::Status;
use toml::Value;
#[macro_use]
extern crate rocket; // use all rocket macros

#[get("/file?<path>&<name_only>")]
fn get_file(path: &str, name_only: bool) -> Result<String, Status> {
    if name_only {
        return Ok(path.split('/').last().unwrap().to_owned());
    }

    let toml_file: Value = fs::read_to_string("lavender.toml")
        .unwrap()
        .parse()
        .unwrap();
    let config = toml_file["config"].as_table().unwrap();
    let root = config["media_path"].as_str().unwrap();
    let filepath = format!("{}/{}", root, path);
    let dir = Path::new(&filepath);
    match file::LavenderFile::new(dir) {
        Ok(f) => Ok(f.read_base64()),
        Err(_) => Err(Status::BadRequest),
    }
}

#[get("/optimize")]
fn create_optimized_images(_key: ApiKey) -> &'static str {
    let toml_file: Value = fs::read_to_string("lavender.toml")
        .unwrap()
        .parse()
        .unwrap();
    let config = toml_file["config"].as_table().unwrap();
    let dir = Path::new(config["media_path"].as_str().unwrap());
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap();
            if LavenderFile::is_image(ext) {
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

#[launch]
fn rocket() -> _ {
    let routes = routes![get_file, create_optimized_images];
    let catchers = catchers![bad_request, unauthorized, not_found, internal_server_error];
    rocket::build().register("/", catchers).mount("/", routes)
}
