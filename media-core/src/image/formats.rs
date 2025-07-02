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

    /// 透明度対応かどうか
    pub fn supports_transparency(self) -> bool {
        matches!(self, Self::Png | Self::Gif | Self::WebP | Self::Ico | Self::Tiff)
    }

    /// ロスレス圧縮かどうか
    pub fn is_lossness(self) -> bool {
        matches!(
            self,
            Self::Png | Self::Gif | Self::Bmp | Self::Ico | Self::Tiff | Self::Tga | Self::Pnm | Self::Farbfeld
        )
    }

    /// HDR対応かどうか
    pub fn supports_hdr(self) -> bool {
        matches!(self, Self::Hdr | Self::OpenExr | Self::Tiff)
    }
}

/// 画像メタデータ
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: SupportedImageFormat,
    pub color_type: ColorType,
    pub bit_depth: u8,
    pub has_alpha: bool,
    pub is_animated: bool,
    pub frame_count: Option<usize>,
    pub file_size: Option<u64>,
}

/// カラータイプ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorType {
    L8,      // グレースケール 8-bit
    L16,     // グレースケール 16-bit
    La8,     // グレースケール + アルファ 8-bit
    La16,    // グレースケール + アルファ 16-bit
    Rgb8,    // RGB 8-bit
    Rgb16,   // RGB 16-bit
    Rgba8,   // RGBA 8-bit
    Rgba16,  // RGBA 16-bit
    Rgb32F,  // RGB 32-bit float
    Rgba32F, // RGBA 32-bit float
}

impl ColorType {
    /// image クレートの ColorType から変換
    pub fn from_image_color_type(color_type: image::ColorType) -> Self {
        match color_type {
            image::ColorType::L8 => Self::L8,
            image::ColorType::L16 => Self::L16,
            image::ColorType::La8 => Self::La8,
            image::ColorType::La16 => Self::La16,
            image::ColorType::Rgb8 => Self::Rgb8,
            image::ColorType::Rgb16 => Self::Rgb16,
            image::ColorType::Rgba8 => Self::Rgba8,
            image::ColorType::Rgba16 => Self::Rgba16,
            image::ColorType::Rgb32F => Self::Rgb32F,
            image::ColorType::Rgba32F => Self::Rgba32F,
            _ => Self::Rgb8, // フォールバック
        }
    }

    /// チャンネル数を取得
    pub fn channel_count(self) -> u8 {
        match self {
            Self::L8 | Self::L16 => 1,
            Self::La8 | Self::La16 => 2,
            Self::Rgb8 | Self::Rgb16 | Self::Rgb32F => 3,
            Self::Rgba8 | Self::Rgba16 | Self::Rgba32F => 4,
        }
    }

    ///  チャンネルあたりのビット数
    pub fn bits_per_channel(self) -> u8 {
        match self {
            Self::L8 | Self::La8 | Self::Rgb8 | Self::Rgba8 => 8,
            Self::L16 | Self::La16 | Self::Rgb16 | Self::Rgba16 => 16,
            Self::Rgb32F | Self::Rgba32F => 32,
        }
    }

    /// アルファチャンネルがあるかどうか
    pub fn has_alpha(self) -> bool {
        matches!(self, Self::La8 | Self::La16 | Self::Rgba8 | Self::Rgba16 | Self::Rgba32F)
    }

    /// 浮動小数点形式かどうか
    pub fn is_float(self) -> bool {
        matches!(self, Self::Rgb32F | Self::Rgba32F)
    }
}