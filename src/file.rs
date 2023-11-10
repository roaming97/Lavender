use serde::Deserialize;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
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
            Some(ext) => DataType::from(ext.to_ascii_lowercase().to_str().unwrap(), None),
            None => DataType::Unknown,
        };
        Self { buffer, datatype }
    }

    /// Ckecks that:
    /// * The file's buffer is not empty.
    /// * The file's data type is unknown.
    pub fn is_valid(&self) -> bool {
        !self.buffer.is_empty() && !self.datatype.is_type(&DataType::Unknown)
    }

    /// Reads a media file and returns an HTML-friendly data `base64` string.
    pub fn read_base64(&self) -> String {
        let engine = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());
        engine.encode(&self.buffer)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataType {
    Image,
    Video,
    Audio,
    Unknown,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::Audio => "Audio",
                Self::Image => "Image",
                Self::Video => "Video",
                Self::Unknown => "Unknown",
            }
        )
    }
}

impl DataType {
    pub fn from<T: AsRef<OsStr>>(input: T, state: Option<&Arc<LavenderConfig>>) -> Self {
        if input.as_ref().is_empty() {
            return Self::Unknown;
        }
        let extension = input.as_ref().to_ascii_lowercase();
        let t = match state {
            Some(s) => {
                let image_exts = &s.extensions.image;
                let video_exts = &s.extensions.video;
                let audio_exts = &s.extensions.audio;

                let search = format!("{}", extension.to_string_lossy());

                // Match against the extension lists
                if image_exts.contains(&search) {
                    Self::Image
                } else if video_exts.contains(&search) {
                    Self::Video
                } else if audio_exts.contains(&search) {
                    Self::Audio
                } else {
                    Self::Unknown
                }
            }
            None => {
                let toml = LavenderTOML::new();
                let extension = Value::String(extension.to_string_lossy().to_string());

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
        };
        if t.is_type(&Self::Unknown) {
            match extension.to_str().unwrap_or_default() {
                "image" => Self::Image,
                "video" => Self::Video,
                "audio" => Self::Audio,
                _ => Self::Unknown,
            }
        } else {
            t
        }
    }
    pub fn is_type(&self, datatype: &DataType) -> bool {
        self.eq(datatype)
    }
}

#[derive(Deserialize)]
pub struct LavenderConfig {
    pub server: ServerConfig,
    pub extensions: ExtensionsConfig,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub media_path: String,
}

#[derive(Deserialize)]
pub struct ExtensionsConfig {
    pub image: Vec<String>,
    pub video: Vec<String>,
    pub audio: Vec<String>,
}

impl LavenderConfig {
    pub fn new() -> Self {
        let toml_str =
            fs::read_to_string("lavender.toml").expect("Failed to read configuration TOML");
        let config: Self =
            toml::from_str(&toml_str).expect("Failed to deserialize configuration TOML.");
        config
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

    pub fn get_array_value(&self, table: &str, value: &str) -> Option<Vec<Value>> {
        let t = self.0[table].as_table()?;
        t[value].as_array().map(|v| v.to_vec())
    }
}

pub fn get_all_files_recursively<P: AsRef<Path>>(root: P) -> Vec<walkdir::DirEntry> {
    WalkDir::new(root)
        .sort_by_file_name()
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
