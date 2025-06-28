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

    /// FFmpeg のフレームから VideoFrame を作成
    pub fn from_ffmpeg_frame(frame: &ffmpeg_next::frame::Video, timestamp: Duration, pts: i64) -> Result<Self> {
        let width = frame.width();
        let height = frame.height();
        let format = Self::convert_ffmpeg_format(frame.format())?;

        // フレームデータをコピー
        let mut data = Vec::new();
        for plane in 0..frame.planes() {
            let plane_data = frame.data(plane);
            data.extend_from_slice(plane_data);
        }

        Ok(Self::new(data, width, height, format, timestamp, pts))
    }

    /// OpenCV の Mat から VideoFrame を作成
    pub fn from_opencv_mat(mat: &Mat, timestamp: Duration, pts: i64) -> Result<Self> {
        let size = mat.size()?;
        let width = size.width as u32;
        let height = size.height as u32;

        // Mat のタイプからフォーマットを指定
        let mat_type = mat.typ();
        let format = match mat_type {
            opencv::core::CV_8UC3 => FrameFormat::BGR8,
            opencv::core::CV_8UC4 => FrameFormat::BGRA8,
            opencv::core::CV_8UC1 => FrameFormat::Gray8,
            _ => return Err(MediaError::Video(format!("Unsupported mat type: {}", mat_type))),
        };

        // データをコピー
        let data = mat.data_bytes()?.to_vec();

        Ok(Self::new(data, width, height, format, timestamp, pts))
    }

    /// FFmpeg のピクセルフォーマットを変換
    fn convert_ffmpeg_format(format: ffmpeg_next::format::Pixel) -> Result<FrameFormat> {
        match format {
            ffmpeg_next::format::Pixel::RGB24 => Ok(FrameFormat::RGB8),
            ffmpeg_next::format::Pixel::RGBA => Ok(FrameFormat::RGBA8),
            ffmpeg_next::format::Pixel::BGR24 => Ok(FrameFormat::BGR8),
            ffmpeg_next::format::Pixel::BGRA => Ok(FrameFormat::BGRA8),
            ffmpeg_next::format::Pixel::YUV420P => Ok(FrameFormat::YUV420P),
            ffmpeg_next::format::Pixel::GRAY8 => Ok(FrameFormat::Gray8),
            _ => Err(MediaError::UnsupportedCodec(format!("Unsupported pixel format: {:?}", format))),
        }
    }
}