use std::time::Duration;

use image::{DynamicImage, ImageBuffer};
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
    pub fn from_ffmpeg_frame(
        frame: &ffmpeg_next::frame::Video,
        timestamp: Duration,
        pts: i64,
    ) -> Result<Self> {
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
            _ => {
                return Err(MediaError::Video(format!(
                    "Unsupported mat type: {}",
                    mat_type
                )));
            }
        };

        // データをコピー
        let data = mat.data_bytes()?.to_vec();

        Ok(Self::new(data, width, height, format, timestamp, pts))
    }

    /// image クレートの DynamicImage に変換
    pub fn to_dynamic_image(&self) -> Result<DynamicImage> {
        match self.format {
            FrameFormat::RGB8 => {
                let img = ImageBuffer::<image::Rgb<u8>, _>::from_raw(
                    self.width,
                    self.height,
                    self.data.clone(),
                )
                .ok_or_else(|| {
                    MediaError::Image(image::ImageError::Parameter(
                        image::error::ParameterError::from_kind(
                            image::error::ParameterErrorKind::DimensionMismatch,
                        ),
                    ))
                })?;
                Ok(DynamicImage::ImageRgb8(img))
            }
            FrameFormat::RGBA8 => {
                let img = ImageBuffer::<image::Rgba<u8>, _>::from_raw(
                    self.width,
                    self.height,
                    self.data.clone(),
                )
                .ok_or_else(|| {
                    MediaError::Image(image::ImageError::Parameter(
                        image::error::ParameterError::from_kind(
                            image::error::ParameterErrorKind::DimensionMismatch,
                        ),
                    ))
                })?;
                Ok(DynamicImage::ImageRgba8(img))
            }
            FrameFormat::BGR8 => {
                // BGR を RGB に変換
                let mut rgb_data = Vec::with_capacity(self.data.len());
                for chunk in self.data.chunks(3) {
                    if chunk.len() == 3 {
                        rgb_data.push(chunk[2]); // R
                        rgb_data.push(chunk[1]); // G
                        rgb_data.push(chunk[0]); // B
                    }
                }
                let img =
                    ImageBuffer::<image::Rgb<u8>, _>::from_raw(self.width, self.height, rgb_data)
                        .ok_or_else(|| {
                        MediaError::Image(image::ImageError::Parameter(
                            image::error::ParameterError::from_kind(
                                image::error::ParameterErrorKind::DimensionMismatch,
                            ),
                        ))
                    })?;
                Ok(DynamicImage::ImageRgb8(img))
            }
            FrameFormat::Gray8 => {
                let img = ImageBuffer::<image::Luma<u8>, _>::from_raw(
                    self.width,
                    self.height,
                    self.data.clone(),
                )
                .ok_or_else(|| {
                    MediaError::Image(image::ImageError::Parameter(
                        image::error::ParameterError::from_kind(
                            image::error::ParameterErrorKind::DimensionMismatch,
                        ),
                    ))
                })?;
                Ok(DynamicImage::ImageLuma8(img))
            }
            _ => Err(MediaError::Video(
                "Unsupported format for conversion to DynamicImage".to_string(),
            )),
        }
    }

    /// OpenCV の Mat に変換
    pub fn to_opencv_mat(&self) -> Result<Mat> {
        let mat_type = match self.format {
            FrameFormat::BGR8 => opencv::core::CV_8UC3,
            FrameFormat::BGRA8 => opencv::core::CV_8UC4,
            FrameFormat::RGB8 => opencv::core::CV_8UC3,
            FrameFormat::RGBA8 => opencv::core::CV_8UC4,
            FrameFormat::Gray8 => opencv::core::CV_8UC1,
            _ => {
                return Err(MediaError::Video(
                    "Unsupported format for OpenCV Mat".to_string(),
                ));
            }
        };

        let mut mat = Mat::new_rows_cols_with_default(
            self.height as i32,
            self.width as i32,
            mat_type,
            opencv::core::Scalar::all(0.0),
        )?;

        // データをコピー
        let mat_data = mat.data_bytes_mut()?;

        if self.format == FrameFormat::RGB8 || self.format == FrameFormat::RGBA8 {
            // RGB から BGR に変換
            let channels = if self.format == FrameFormat::RGB8 {
                3
            } else {
                4
            };
            for i in 0..(self.data.len() / channels) {
                let base_idx = i * channels;
                mat_data[base_idx] = self.data[base_idx + 2]; // B
                mat_data[base_idx + 1] = self.data[base_idx + 1]; // G
                mat_data[base_idx + 2] = self.data[base_idx]; // R
                if channels == 4 {
                    mat_data[base_idx + 3] = self.data[base_idx + 3]; // A
                }
            }
        } else {
            mat_data.copy_from_slice(&self.data);
        }

        Ok(mat)
    }

    /// リサイズ
    pub fn resize(&self, new_width: u32, new_height: u32) -> Result<Self> {
        let dynamic_img = self.to_dynamic_image()?;
        let resized =
            dynamic_img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

        // リサイズされた画像から VideoFrame を作成
        match resized {
            DynamicImage::ImageRgb8(img) => Ok(Self::new(
                img.into_raw(),
                new_width,
                new_height,
                FrameFormat::RGB8,
                self.timestamp,
                self.pts,
            )),
            DynamicImage::ImageRgba8(img) => Ok(Self::new(
                img.into_raw(),
                new_width,
                new_height,
                FrameFormat::RGBA8,
                self.timestamp,
                self.pts,
            )),
            DynamicImage::ImageLuma8(img) => Ok(Self::new(
                img.into_raw(),
                new_width,
                new_height,
                FrameFormat::Gray8,
                self.timestamp,
                self.pts,
            )),
            _ => Err(MediaError::Video(
                "Unsupported image format after resize".to_string(),
            )),
        }
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
            _ => Err(MediaError::UnsupportedCodec(format!(
                "Unsupported pixel format: {:?}",
                format
            ))),
        }
    }
}
