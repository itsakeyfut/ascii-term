use std::collections::VecDeque;

use opencv::{core::Mat, prelude::*, imgproc};

use crate::errors::{MediaError, Result};
use crate::video::frame::{VideoFrame, FrameFormat};

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