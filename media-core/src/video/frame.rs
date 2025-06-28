use std::time::Duration;

use image::{ImageBuffer, RgbImage, RgbaImage, DynamicImage};
use opencv::{core::Mat, prelude::*};

use crate::errors::{MediaError, Result};

/// フレームのピクセルフォーマット
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameFormat {
    RGB8,
    RGBA8,
    BGR8,
    BGRA8,
    YUV420P,
    Gray8,
}
