use std::time::Duration;

use image::{DynamicImage, ImageBuffer};

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

    /// avio の VideoFrame から VideoFrame を作成
    pub fn from_avio_frame(frame: &avio::VideoFrame) -> Result<Self> {
        let width = frame.width();
        let height = frame.height();
        let format = Self::convert_avio_format(frame.format())?;
        let timestamp = frame.timestamp().as_duration();
        let pts = frame.timestamp().pts();
        let data = frame.data();

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

    /// リサイズ
    pub fn resize(&self, new_width: u32, new_height: u32) -> Result<Self> {
        let dynamic_img = self.to_dynamic_image()?;
        let resized =
            dynamic_img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

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

    /// avio のピクセルフォーマットを変換
    fn convert_avio_format(format: avio::PixelFormat) -> Result<FrameFormat> {
        match format {
            avio::PixelFormat::Rgb24 => Ok(FrameFormat::RGB8),
            avio::PixelFormat::Rgba => Ok(FrameFormat::RGBA8),
            avio::PixelFormat::Bgr24 => Ok(FrameFormat::BGR8),
            avio::PixelFormat::Bgra => Ok(FrameFormat::BGRA8),
            avio::PixelFormat::Yuv420p => Ok(FrameFormat::YUV420P),
            avio::PixelFormat::Gray8 => Ok(FrameFormat::Gray8),
            _ => Err(MediaError::UnsupportedCodec(format!(
                "Unsupported pixel format: {:?}",
                format
            ))),
        }
    }
}
