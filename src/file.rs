use std::path::Path;

use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::{alphabet, Engine};

pub struct LavenderFile {
    buffer: Vec<u8>,
    extension: String,
}

impl LavenderFile {
    pub const IMAGE_EXTS: [&str; 6] = ["jpg", "jpeg", "png", "gif", "bmp", "webp"];
    pub const VIDEO_EXTS: [&str; 6] = ["mp4", "mov", "avi", "mkv", "wmv", "webm"];

    pub fn is_image(ext: &str) -> bool {
        Self::IMAGE_EXTS.contains(&ext)
    }

    /// Creates a new media file.
    pub fn new(path: &Path) -> Result<Self, String> {
        let buffer = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                return Err(format!("File \'{}\' not found!: {}", path.to_string_lossy(), e));
            }
        };

        if let Some(ext) = path.extension() {
            let extension = ext.to_string_lossy().to_ascii_lowercase();
            if Self::IMAGE_EXTS.contains(&extension.as_str())
                || Self::VIDEO_EXTS.contains(&extension.as_str())
            {
                return Ok(Self { extension, buffer });
            }
        }
        Err("Invalid file extension!".to_string())
    }

    /// Reads a media file and returns an HTML-friendly `base64` string.
    pub fn read_base64(&self) -> String {
        let engine = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());
        format!(
            "data:image/{};base64,{}",
            self.extension,
            engine.encode(&self.buffer)
        )
    }
}
