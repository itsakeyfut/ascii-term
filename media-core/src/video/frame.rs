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

/// ビデオフレームを表現する構造体
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: FrameFormat,
    pub timestamp: Duration,
    pub pts: i64,
}
