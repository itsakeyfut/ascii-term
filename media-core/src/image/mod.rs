pub mod processor;
pub mod formats;

pub use processor::ImageProcessor;
pub use formats::*;

use std::path::Path;

use image::{DynamicImage, ImageFormat};

use crate::errors::{MediaError, Result};

/// 画像ファイルを開く
pub fn open_image<P: AsRef<Path>>(path: P) -> Result<DynamicImage> {
    let image = image::open(path)
        .map_err(|e| MediaError::Image(e))?;
    Ok(image)
}

/// 画像形式を推測
pub fn guess_format<P: AsRef<Path>>(path: P) -> Option<ImageFormat> {
    image::ImageFormat::from_path(path).ok()
}

/// サポートされている画像形式かチェック
pub fn is_supported_format<P: AsRef<Path>>(path: P) -> bool {
    guess_format(path).is_some()
}