use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::{alphabet, Engine};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use toml::Value;
use walkdir::WalkDir;

// pub const MASTER_IMAGE_SUFFIX: &str = "_master.";
// pub const VIDEO_THUMBNAIL_SUFFIX: &str = "_thumb.";

#[derive(Debug, Serialize, Deserialize)]
pub struct LavenderFile {
    pub path: PathBuf,
    pub b64: String,
    datatype: DataType,
    filename: String,
    date: SystemTime,
}

impl LavenderFile {
    /// Creates a new media file.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let buffer = fs::read(&path)?;
        let datatype = match path.extension() {
            Some(ext) => DataType::from(ext.to_ascii_lowercase().to_str().unwrap(), None),
            None => DataType::Unknown,
        };
        let b64 = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default())
            .encode(buffer);
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let date = path.metadata()?.modified()?;

        Ok(Self {
            path,
            datatype,
            b64,
            filename,
            date,
        })
    }

    /// Checks that:
    /// * The file's data type is known.
    /// * The file's base64 data is not empty.
    /// * The file's has a name.
    pub fn is_valid(&self) -> bool {
        !self.datatype.is_type(&DataType::Unknown)
            && !self.b64.is_empty()
            && !self.filename.is_empty()
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum DataType {
    Image,
    Video,
    Audio,
    Unknown,
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
    pub address: String,
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

pub fn scan_fs<P: AsRef<Path>>(root: P, recursive: bool) -> Vec<walkdir::DirEntry> {
    WalkDir::new(root)
        .sort_by(|a, b| {
            let a = a.metadata().unwrap().modified().unwrap();
            let b = b.metadata().unwrap().modified().unwrap();
            b.cmp(&a)
        })
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some())
        .filter(|e| if recursive { true } else { e.depth() <= 1 })
        .collect()
}
