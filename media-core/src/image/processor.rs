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

/// 画像プロセッサー
pub struct ImageProcessor {
    config: ImageProcessorConfig,
    resizer: fr::Resizer,
}

impl ImageProcessor {
    /// 新しい画像プロセッサーを作成
    pub fn new(config: ImageProcessorConfig) -> Self {
        Self {
            config,
            resizer: fr::Resizer::new(),
        }
    }

    /// 設定を更新
    pub fn update_config(&mut self, config: ImageProcessorConfig) {
        self.config = config;
    }

    /// 画像をリサイズ
    pub fn resize(
        &mut self,
        image: &DynamicImage,
        width: u32,
        height: u32,
        algorithm: Option<ResizeAlgorithm>,
    ) -> Result<DynamicImage> {
        let alg = algorithm.unwrap_or(self.config.resize_algorithm);
        
        if image.width() == width && image.height() == height {
            return Ok(image.clone());
        }

        // RGB画像に変換
        let rgb_image = image.to_rgb8();
        
        let src_image = fr::images::Image::from_vec_u8(
            image.width(),
            image.height(),
            rgb_image.into_raw(),
            fr::PixelType::U8x3,
        ).map_err(|e| MediaError::Image(
            image::ImageError::Parameter(
                image::error::ParameterError::from_kind(
                    image::error::ParameterErrorKind::Generic(format!("FastImageResize error: {:?}", e))
                )
            )
        ))?;

        let mut dst_image = fr::images::Image::new(width, height, fr::PixelType::U8x3);

        self.resizer.resize(
            &src_image,
            &mut dst_image,
            &fr::ResizeOptions::new().resize_alg(alg.into()),
        ).map_err(|e| MediaError::Image(
            image::ImageError::Parameter(
                image::error::ParameterError::from_kind(
                    image::error::ParameterErrorKind::Generic(format!("Resize error: {:?}", e))
                )
            )
        ))?;

        let resized_data = dst_image.into_vec();
        let resized_buffer = ImageBuffer::from_raw(width, height, resized_data)
            .ok_or_else(|| MediaError::Image(
                image::ImageError::Parameter(
                    image::error::ParameterError::from_kind(
                        image::error::ParameterErrorKind::DimensionMismatch
                    )
                )
            ))?;

        Ok(DynamicImage::ImageRgb8(resized_buffer))
    }
}