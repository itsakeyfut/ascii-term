use std::path::Path;

use image::{DynamicImage, ImageFormat};

use crate::errors::{MediaError, Result};

/// サポートされている画像形式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SupportedImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Bmp,
    Ico,
    Tiff,
    Tga,
    Dds,
    Hdr,
    OpenExr,
    Pnm,
    Farbfeld,
}

impl SupportedImageFormat {
    /// ファイル拡張子から形式を推測
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "gif" => Some(Self::Gif),
            "webp" => Some(Self::WebP),
            "bmp" => Some(Self::Bmp),
            "ico" => Some(Self::Ico),
            "tiff" | "tif" => Some(Self::Tiff),
            "tga" => Some(Self::Tga),
            "dds" => Some(Self::Dds),
            "hdr" => Some(Self::Hdr),
            "exr" => Some(Self::OpenExr),
            "ppm" | "pgm" | "pbm" | "pam" => Some(Self::Pnm),
            "ff" => Some(Self::Farbfeld),
            _ => None,
        }
    }

    /// ファイルパスから形式を推測
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }
}