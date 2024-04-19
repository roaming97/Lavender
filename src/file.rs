use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::{alphabet, Engine};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct LavenderEntry {
    path: PathBuf,
    b64: String,
    mimetype: String,
    filename: String,
    date: SystemTime,
}

impl LavenderEntry {
    /// Creates a new media file.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mimetype = infer::get_from_path(path.clone())
            .map_or(String::new(), |t| t.unwrap().mime_type().to_owned());
        let buffer = fs::read(&path)?;
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
            b64,
            mimetype,
            filename,
            date,
        })
    }

    #[cfg(test)]
    pub fn base64(&self) -> String {
        self.b64.clone()
    }

    /// Checks that:
    /// * The file's MIME type is supported.
    /// * The file's base64 data is not empty.
    /// * The file's has a name.
    pub fn is_valid(&self) -> bool {
        infer::is_mime_supported(&self.mimetype)
            && !self.b64.is_empty()
            && !self.filename.is_empty()
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
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some())
        .filter(|e| if recursive { true } else { e.depth() <= 1 })
        .collect()
}
