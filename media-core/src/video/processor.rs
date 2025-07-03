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

    /// ガウシアンブラーを適用
    fn apply_gaussian_blur(&self, frame: VideoFrame, kernel_size: i32, sigma: f64) -> Result<VideoFrame> {
        let mat = frame.to_opencv_mat()?;
        let mut blurred = Mat::default();

        imgproc::gaussian_blur(
            &mat,
            &mut blurred,
            opencv::core::Size::new(kernel_size, kernel_size),
            sigma,
            sigma,
            opencv::core::BORDER_DEFAULT,
        )?;

        VideoFrame::from_opencv_mat(&blurred, frame.timestamp, frame.pts)
    }

    /// エッジ検出を適用
    fn apply_edge_detection(&self, frame: VideoFrame, threshold1: f64, threshold2: f64) -> Result<VideoFrame> {
        let mat = frame.to_opencv_mat()?;

        // グレースケールに変換
        let mut gray = Mat::default();
        if frame.format != FrameFormat::Gray8 {
            imgproc::cvt_color(&mat, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;
        } else {
            gray = mat;
        }

        // Cannyエッジ検出
        let mut edges = Mat::default();
        imgproc::canny(&gray, &mut edges, threshold1, threshold2, 3, false)?;

        VideoFrame::from_opencv_mat(&edges, frame.timestamp, frame.pts)
    }

    /// ヒストグラム均一化を適用
    fn apply_histogram_equalization(&self, frame: VideoFrame) -> Result<VideoFrame> {
        let mat = frame.to_opencv_mat()?;
        
        let result_mat = if frame.format == FrameFormat::Gray8 {
            // グレースケール画像の場合
            let mut equalized = Mat::default();
            imgproc::equalize_hist(&mat, &mut equalized)?;
            equalized
        } else {
            // カラー画像の場合はYUVに変換してY成分のみ均一化
            let mut yuv = Mat::default();
            imgproc::cvt_color(&mat, &mut yuv, imgproc::COLOR_BGR2YUV, 0)?;
            
            let mut channels = opencv::core::Vector::<Mat>::new();
            opencv::core::split(&yuv, &mut channels)?;
            
            let mut y_equalized = Mat::default();
            imgproc::equalize_hist(&channels.get(0)?, &mut y_equalized)?;
            
            channels.set(0, y_equalized)?;
            
            let mut yuv_equalized = Mat::default();
            opencv::core::merge(&channels, &mut yuv_equalized)?;
            
            let mut bgr_equalized = Mat::default();
            imgproc::cvt_color(&yuv_equalized, &mut bgr_equalized, imgproc::COLOR_YUV2BGR, 0)?;
            bgr_equalized
        };

        VideoFrame::from_opencv_mat(&result_mat, frame.timestamp, frame.pts)
    }

    /// 色調整を適用
    fn apply_color_adjustment(&self, frame: VideoFrame, brightness: f32, contrast: f32, _saturation: f32) -> Result<VideoFrame> {
        let mat = frame.to_opencv_mat()?;
        let mut adjusted = Mat::default();

        // OpenCVのconvertToを使用して、ブライトネス・コントラスト調整
        mat.convert_to(&mut adjusted, -1, contrast as f64, brightness as f64)?;

        VideoFrame::from_opencv_mat(&adjusted, frame.timestamp, frame.pts)
    }

    /// 回転を適用
    fn apply_rotation(&self, frame: VideoFrame, angle: f64) -> Result<VideoFrame> {
        let mat = frame.to_opencv_mat()?;
        let center = opencv::core::Point2f::new(
            frame.width as f32 / 2.0,
            frame.height as f32 / 2.0,
        );

        let rotation_matrix = imgproc::get_rotation_matrix_2d(center, angle, 1.0)?;
        let mut rotated = Mat::default();

        imgproc::warp_affine(
            &mat,
            &mut rotated,
            &rotation_matrix,
            opencv::core::Size::new(frame.width as i32, frame.height as i32),
            imgproc::INTER_LINEAR,
            opencv::core::BORDER_CONSTANT,
            opencv::core::Scalar::default(),
        )?;

        VideoFrame::from_opencv_mat(&rotated, frame.timestamp, frame.pts)
    }
}