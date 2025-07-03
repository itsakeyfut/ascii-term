use std::collections::VecDeque;

use opencv::{core::Mat, prelude::*, imgproc};

use crate::errors::{MediaError, Result};
use crate::video::frame::{VideoFrame, FrameFormat};

/// ビデオ処理設定
#[derive(Debug, Clone)]
pub struct VideoProcessorConfig {
    /// バッファサイズ
    pub buffer_size: usize,
    /// デフォルトの出力形式
    pub output_format: FrameFormat,
    /// フィルターチェーン
    pub filter_chain: Vec<VideoFilter>,
}

impl Default for VideoProcessorConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10,
            output_format: FrameFormat::RGB8,
            filter_chain: Vec::new(),
        }
    }
}

/// ビデオフィルター
#[derive(Debug, Clone)]
pub enum VideoFilter {
    /// リサイズ
    Resize { width: u32, height: u32 },
    /// 色空間変換
    ColorConvert { target_format: FrameFormat },
    /// ガウシアンブラー
    GaussianBlur { kernel_size: i32, sigma: f64 },
    /// エッジ検出
    EdgeDetection { threshold: f64, threshold2: f64 },
    /// ヒストグラム均一化
    HistogramEqualization,
    /// 色調整
    ColorAdjust { brightness: f32, contrast: f32, saturation: f32 },
    /// 回転
    Flip { horizontal: bool, vertical: bool },
    /// クロップ
    Crop { x: u32, y: u32, width: u32, height: u32 },
}

/// ビデオプロセッサー
pub struct VideoProcessor {
    config: VideoProcessorConfig,
    buffer: VecDeque<VideoFrame>,
}

impl VideoProcessor {
    /// 新しいビデオプロセッサーを作成
    pub fn new(config: VideoProcessorConfig) -> Self {
        Self {
            config,
            buffer: VecDeque::new(),
        }
    }

    /// 設定を更新
    pub fn update_config(&mut self, config: VideoProcessorConfig) {
        self.config = config;
    }

    /// 形式変換
    fn convert_format(&self, frame: VideoFrame, target_format: FrameFormat) -> Result<VideoFrame> {
        if frame.format == target_format {
            return Ok(frame);
        }

        let mat = frame.to_opencv_mat()?;
        let converted_mat = match (frame.format, target_format) {
            (FrameFormat::BGR8, FrameFormat::RGB8) => {
                let mut rgb_mat = Mat::default();
                imgproc::cvt_color(&mat, &mut rgb_mat, imgproc::COLOR_BGR2RGB, 0)?;
                rgb_mat
            }
            (FrameFormat::RGB8, FrameFormat::BGR8) => {
                let mut bgr_mat = Mat::default();
                imgproc::cvt_color(&mat, &mut bgr_mat, imgproc::COLOR_RGB2BGR, 0)?;
                bgr_mat
            }
            (FrameFormat::BGR8, FrameFormat::Gray8) => {
                let mut gray_mat = Mat::default();
                imgproc::cvt_color(&mat, &mut gray_mat, imgproc::COLOR_BGR2GRAY, 0)?;
                gray_mat
            }
            (FrameFormat::RGB8, FrameFormat::Gray8) => {
                let mut gray_mat = Mat::default();
                imgproc::cvt_color(&mat, &mut gray_mat, imgproc::COLOR_RGB2GRAY, 0)?;
                gray_mat
            }
            _ => return Err(MediaError::Video("Unsupported format conversion".to_string())),
        };

        VideoFrame::from_opencv_mat(&converted_mat, frame.timestamp, frame.pts)
    }
}