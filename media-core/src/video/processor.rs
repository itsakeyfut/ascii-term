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
    EdgeDetection { threshold1: f64, threshold2: f64 },
    /// ヒストグラム均一化
    HistogramEqualization,
    /// 色調整
    ColorAdjust { brightness: f32, contrast: f32, saturation: f32 },
    /// 回転
    Rotate { angle: f64 },
    /// 反転
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

    /// フレームを処理
    pub fn process_frame(&mut self, frame: VideoFrame) -> Result<()> {
        let mut processed_frame = frame;

        // フィルターチェーンを適用
        for filter in &self.config.filter_chain {
            processed_frame = self.apply_filter(processed_frame, filter)?;
        }

        // 出力形式に変換（必要に応じて）
        if processed_frame.format != self.config.output_format {
            processed_frame = self.convert_format(processed_frame, self.config.output_format)?;
        }

        // バッファに追加
        self.buffer.push_back(processed_frame);

        // バッファサイズを制限
        while self.buffer.len() > self.config.buffer_size {
            self.buffer.pop_front();
        }

        Ok(())
    }

    /// 次のフレームを取得
    pub fn next_frame(&mut self) -> Option<VideoFrame> {
        self.buffer.pop_front()
    }

    /// フィルターを適用
    fn apply_filter(&self, frame: VideoFrame, filter: &VideoFilter) -> Result<VideoFrame> {
        match filter {
            VideoFilter::Resize { width, height } => {
                frame.resize(*width, *height)
            }
            VideoFilter::ColorConvert { target_format } => {
                self.convert_format(frame, *target_format)
            }
            VideoFilter::GaussianBlur { kernel_size, sigma } => {
                self.apply_gaussian_blur(frame, *kernel_size, *sigma)
            }
            VideoFilter::EdgeDetection { threshold1, threshold2 } => {
                self.apply_edge_detection(frame, *threshold1, *threshold2)
            }
            VideoFilter::HistogramEqualization => {
                self.apply_histogram_equalization(frame)
            }
            VideoFilter::ColorAdjust { brightness, contrast, saturation } => {
                self.apply_color_adjustment(frame, *brightness, *contrast, *saturation)
            }
            VideoFilter::Rotate { angle } => {
                self.apply_rotation(frame, *angle)
            }
            VideoFilter::Flip { horizontal, vertical } => {
                self.apply_flip(frame, *horizontal, *vertical)
            }
            VideoFilter::Crop { x, y, width, height } => {
                self.apply_crop(frame, *x, *y, *width, *height)
            }
        }
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

    /// 反転を適用
    fn apply_flip(&self, frame: VideoFrame, horizontal: bool, vertical: bool) -> Result<VideoFrame> {
        let mat = frame.to_opencv_mat()?;
        let mut flipped = mat.clone();

        if horizontal && vertical {
            opencv::core::flip(&mat, &mut flipped, -1)?; // 両方向
        } else if horizontal {
            opencv::core::flip(&mat, &mut flipped, 1)?; // 水平
        } else if vertical {
            opencv::core::flip(&mat, &mut flipped, 0)?; // 垂直
        }

        VideoFrame::from_opencv_mat(&flipped, frame.timestamp, frame.pts)
    }

    /// クロップを適用
    fn apply_crop(&self, frame: VideoFrame, x: u32, y: u32, width: u32, height: u32) -> Result<VideoFrame> {
        let mat = frame.to_opencv_mat()?;
        
        // 境界チェック
        let crop_x = x.min(frame.width - 1) as i32;
        let crop_y = y.min(frame.height - 1) as i32;
        let crop_width = width.min(frame.width - x) as i32;
        let crop_height = height.min(frame.height - y) as i32;
        
        let roi = opencv::core::Rect::new(crop_x, crop_y, crop_width, crop_height);
        let cropped = Mat::roi(&mat, roi)?;

        VideoFrame::from_opencv_mat(&cropped.clone_pointee(), frame.timestamp, frame.pts)
    }
}