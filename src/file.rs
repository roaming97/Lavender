use num_traits::{FromPrimitive, PrimInt};
use std::fs;
use std::path::{Path, MAIN_SEPARATOR_STR};
use std::sync::Arc;
use toml::Value;

use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::{alphabet, Engine};
use walkdir::WalkDir;

pub const MASTER_FILE_SUFFIX: &str = "_master.";

pub struct LavenderFile {
    buffer: Vec<u8>,
    datatype: DataType,
}

impl LavenderFile {
    /// Creates a new media file.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
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

#[derive(Debug, PartialEq)]
pub enum DataType {
    Image,
    Video,
    Audio,
    Unknown,
}

impl DataType {
    pub fn from_extension(extension: &str) -> Self {
        let toml = LavenderTOML::new();
        let extension = Value::from(extension);

        let image_exts = toml.get_array_value("extensions", "image").unwrap();
        let video_exts = toml.get_array_value("extensions", "video").unwrap();
        let audio_exts = toml.get_array_value("extensions", "audio").unwrap();

        // Match against the extension lists
        if image_exts.contains(&extension) {
            Self::Image
        } else if video_exts.contains(&extension) {
            Self::Video
        } else if audio_exts.contains(&extension) {
            Self::Audio
        } else {
            Self::Unknown
        }
    }

    pub fn from_state(extension: &str, state: &Arc<LavenderConfig>) -> Self {
        let image_exts = &state.image_exts;
        let video_exts = &state.video_exts;
        let audio_exts = &state.audio_exts;

        // Match against the extension lists
        if image_exts.contains(&extension.to_owned()) {
            Self::Image
        } else if video_exts.contains(&extension.to_owned()) {
            Self::Video
        } else if audio_exts.contains(&extension.to_owned()) {
            Self::Audio
        } else {
            Self::Unknown
        }
    }

    pub fn is_type(&self, datatype: DataType) -> bool {
        self.eq(&datatype)
    }
}

pub struct LavenderConfig {
    pub port: u16,
    pub media_path: String,

    pub image_exts: Vec<String>,
    pub video_exts: Vec<String>,
    pub audio_exts: Vec<String>,
}

impl LavenderConfig {
    pub fn new() -> Self {
        let toml = LavenderTOML::new();
        let media_path = toml
            .get_string_value("config", "media_path")
            .unwrap()
            .replace('/', MAIN_SEPARATOR_STR);

        let port = toml.get_number_value::<u16>("config", "port").unwrap();

        let image_exts: Vec<String> = toml
            .get_array_value("extensions", "image")
            .unwrap()
            .iter()
            .map(|v| v.to_string())
            .collect();
        let video_exts: Vec<String> = toml
            .get_array_value("extensions", "video")
            .unwrap()
            .iter()
            .map(|v| v.to_string())
            .collect();
        let audio_exts: Vec<String> = toml
            .get_array_value("extensions", "audio")
            .unwrap()
            .iter()
            .map(|v| v.to_string())
            .collect();

        Self {
            port,
            media_path,
            image_exts,
            video_exts,
            audio_exts,
        }
    }
}
pub struct LavenderTOML(Value);

impl LavenderTOML {
    pub fn new() -> Self {
        let toml_file: Value = fs::read_to_string("lavender.toml")
            .unwrap()
            .parse()
            .unwrap();
        Self(toml_file)
    }

    pub fn get_string_value(&self, table: &str, value: &str) -> Option<String> {
        let t = self.0[table].as_table().unwrap();
        t[value].as_str().map(|v| v.to_owned())
    }

    #[allow(dead_code)]
    pub fn get_number_value<T: PrimInt + FromPrimitive>(
        &self,
        table: &str,
        value: &str,
    ) -> Option<T> {
        let t = self.0[table].as_table().unwrap();
        match t[value].as_integer() {
            Some(n) => FromPrimitive::from_i64(n),
            None => None,
        }
    }

    pub fn get_array_value(&self, table: &str, value: &str) -> Option<Vec<Value>> {
        let t = self.0[table].as_table().unwrap();
        t[value].as_array().map(|v| v.to_vec())
    }
}

pub fn get_all_files_recursively(state: &Arc<LavenderConfig>) -> Vec<walkdir::DirEntry> {
    WalkDir::new(&state.media_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().is_some()
                && e.file_name()
                    .to_str()
                    .map(|s| !s.starts_with('.') && !s.contains(MASTER_FILE_SUFFIX))
                    .unwrap_or(false)
        })
        .collect()
}
