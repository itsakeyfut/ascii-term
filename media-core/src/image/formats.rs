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
    Farbfelg,
}
