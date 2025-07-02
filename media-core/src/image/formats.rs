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

    /// image クレートの ImageFormat に変換
    pub fn to_image_format(self) -> Option<ImageFormat> {
        match self {
            Self::Png => Some(ImageFormat::Png),
            Self::Jpeg => Some(ImageFormat::Jpeg),
            Self::Gif => Some(ImageFormat::Gif),
            Self::WebP => Some(ImageFormat::WebP),
            Self::Bmp => Some(ImageFormat::Bmp),
            Self::Ico => Some(ImageFormat::Ico),
            Self::Tiff => Some(ImageFormat::Tiff),
            Self::Tga => Some(ImageFormat::Tga),
            Self::Dds => Some(ImageFormat::Dds),
            Self::Hdr => Some(ImageFormat::Hdr),
            Self::OpenExr => Some(ImageFormat::OpenExr),
            Self::Pnm => Some(ImageFormat::Pnm),
            Self::Farbfeld => Some(ImageFormat::Farbfeld),
        }
    }

    /// アニメーション対応かどうか
    pub fn supports_animation(self) -> bool {
        matches!(self, Self::Gif | Self::WebP)
    }
}