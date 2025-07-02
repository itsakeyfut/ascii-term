use image::{DynamicImage, ImageBuffer, Rgb, Rgba, Luma};
use fast_image_resize as fr;

use crate::errors::{MediaError, Result};

/// リサイズアルゴリズム
pub enum ResizeAlgorithm {
    Nearest,
    Bilinear,
    Bicubic,
    Lanczos3,
    CatmullRom,
}
