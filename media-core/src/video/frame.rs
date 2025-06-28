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

impl VideoFrame {
    /// 新しいビデオフレームを作成
    pub fn new(
        data: Vec<u8>,
        width: u32,
        height: u32,
        format: FrameFormat,
        timestamp: Duration,
        pts: i64,
    ) -> Self {
        Self {
            data,
            width,
            height,
            format,
            timestamp,
            pts,
        }
    }
}