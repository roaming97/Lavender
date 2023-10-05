use std::fs;
use std::path::{Path, MAIN_SEPARATOR_STR};
use toml::Value;

use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::{alphabet, Engine};

pub const MASTER_FILE_SUFFIX: &str = "_master.";

pub struct LavenderFile {
    buffer: Vec<u8>,
    datatype: DataType,
}

#[derive(Debug, PartialEq)]
pub enum DataType {
    Image,
    Video,
    Audio,
    Unknown,
}

#[allow(dead_code)]
impl DataType {
    pub fn from_extension(extension: &str) -> Self {
        // Read the TOML file
        let toml_str = fs::read_to_string("lavender.toml").unwrap();
        let toml: Value = toml::from_str(&toml_str).unwrap();

        // Extract the extension lists from the TOML file
        let image_exts = toml["extensions"]["image"].as_array().unwrap();
        let video_exts = toml["extensions"]["video"].as_array().unwrap();
        let audio_exts = toml["extensions"]["audio"].as_array().unwrap();

        // Match against the extension lists
        if image_exts.contains(&Value::from(extension)) {
            Self::Image
        } else if video_exts.contains(&Value::from(extension)) {
            Self::Video
        } else if audio_exts.contains(&Value::from(extension)) {
            Self::Audio
        } else {
            Self::Unknown
        }
    }

    pub fn is_type(&self, datatype: DataType) -> bool {
        self.eq(&datatype)
    }
}

impl LavenderFile {
    /// Creates a new media file.
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let buffer = fs::read(&path).ok().unwrap_or_default();
        let datatype = match path.as_ref().extension() {
            Some(ext) => DataType::from_extension(ext.to_ascii_lowercase().to_str().unwrap()),
            None => DataType::Unknown,
        };
        Self { buffer, datatype }
    }

    /// Ckecks that:
    /// * The file's buffer is not empty.
    /// * The file's data type is unknown.
    pub fn is_valid(&self) -> bool {
        !self.buffer.is_empty() && !self.datatype.is_type(DataType::Unknown)
    }

    /// Reads a media file and returns an HTML-friendly data `base64` string.
    pub fn read_base64(&self) -> String {
        let engine = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());
        engine.encode(&self.buffer)
    }
}

pub fn get_media_path() -> String {
    let toml_file: Value = fs::read_to_string("lavender.toml")
        .unwrap()
        .parse()
        .unwrap();
    let config = toml_file["config"].as_table().unwrap();

    config["media_path"]
        .as_str()
        .unwrap()
        .replace('/', MAIN_SEPARATOR_STR)
}
