use image::{DynamicImage, ImageBuffer, Rgb, Rgba, Luma};
use fast_image_resize as fr;

use crate::errors::{MediaError, Result};

/// 画像処理設定
#[derive(Debug, Clone)]
pub struct ImageProcessorConfig {
    /// デフォルトのリサイズアルゴリズム
    pub resize_algorithm: ResizeAlgorithm,
    /// 品質設定
    pub quality: ImageQuality,
    /// カラープロファイル
    pub color_profile: ColorProfile,
}

impl Default for ImageProcessorConfig {
    fn default() -> Self {
        Self {
            resize_algorithm: ResizeAlgorithm::Lanczos3,
            quality: ImageQuality::High,
            color_profile: ColorProfile::SRGB,
        }
    }
}

/// リサイズアルゴリズム
#[derive(Debug, Clone, Copy)]
pub enum ResizeAlgorithm {
    Nearest,
    Bilinear,
    Bicubic,
    Lanczos3,
    CatmullRom,
}

impl From<ResizeAlgorithm> for fr::ResizeAlg {
    fn from(alg: ResizeAlgorithm) -> Self {
        match alg {
            ResizeAlgorithm::Nearest => fr::ResizeAlg::Nearest,
            ResizeAlgorithm::Bilinear => unimplemented!(),
            ResizeAlgorithm::Bicubic => unimplemented!(),
            ResizeAlgorithm::Lanczos3 => unimplemented!(),
            ResizeAlgorithm::CatmullRom => unimplemented!(),
        }
    }
}

/// 画質設定
#[derive(Debug, Clone, Copy)]
pub enum ImageQuality {
    Low,
    Medium,
    High,
    Maximum,
}

/// カラープロファイル
#[derive(Debug, Clone, Copy)]
pub enum ColorProfile {
    SRGB,
    Linear,
    Rec709,
}

/// 画像フィルター
#[derive(Debug, Clone)]
pub enum ImageFilter {
    /// ブライトネス調整（-1.0 to 1.0）
    Brightness(f32),
    /// コントラスト調整（0.0 to 2.0）
    Contrast(f32),
    /// 彩度調整（0.0 to 2.0）
    Saturation(f32),
    /// ガンマ補正（0.1 to 3.0）
    Gamma(f32),
    /// ガウシアンブラー
    GaussianBlur(f32),
    /// シャープネス
    Sharpen(f32),
    /// グレースケール変換
    Grayscale,
    /// セピア調
    Sepia,
    /// ネガティブ（反転）
    Invert,
}
