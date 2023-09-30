use std::fs;
use std::path::{Path, MAIN_SEPARATOR_STR};
use toml::Value;

use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::{alphabet, Engine};

pub const MASTER_FILE_SUFFIX: &str = "_master.";

#[derive(Debug)]
pub struct LavenderFile {
    buffer: Vec<u8>,
    extension: String,
    pub datatype: DataType,
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

    fn is_type(&self, datatype: DataType) -> bool {
        self.eq(&datatype)
    }

    /// Determines if the data type belongs to an image file.
    pub fn is_image(&self) -> bool {
        self.is_type(DataType::Image)
    }

    /// Determines if the data type belongs to an audio file.
    pub fn is_audio(&self) -> bool {
        self.is_type(DataType::Audio)
    }

    /// Determines if the data type belongs to a video file.
    pub fn is_video(&self) -> bool {
        self.is_type(DataType::Video)
    }

    /// Determines if the data type is unknown.
    pub fn is_unknown(&self) -> bool {
        self.is_type(DataType::Unknown)
    }

    fn get_name(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Unknown => "unknown",
        }
    }
}

impl LavenderFile {
    /// Creates a new media file.
    pub fn new(path: &Path) -> Result<Self, String> {
        let buffer = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                return Err(format!(
                    "Could not read file \'{}\': {}",
                    path.to_string_lossy(),
                    e
                ));
            }
        };

        if let Some(ext) = path.extension() {
            let extension = ext.to_string_lossy().to_ascii_lowercase();
            let datatype = DataType::from_extension(extension.as_str());
            return Ok(Self {
                extension,
                datatype,
                buffer,
            });
        };
        Err("Invalid file extension!".to_string())
    }

    /// Reads a media file and returns an HTML-friendly data `base64` string.
    pub fn read_base64(&self) -> String {
        let engine = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());
        format!(
            "data:{}/{};base64,{}",
            self.datatype.get_name(),
            self.extension,
            engine.encode(&self.buffer)
        )
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
