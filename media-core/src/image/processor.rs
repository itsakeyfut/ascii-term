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

    /// アスペクト比を保持してリサイズ
    pub fn resize_preserve_aspect(
        &mut self,
        image: &DynamicImage,
        max_width: u32,
        max_height: u32,
        algorithm: Option<ResizeAlgorithm>,
    ) -> Result<DynamicImage> {
        let (orig_width, orig_height) = (image.width(), image.height());

        let scale_x = max_width as f64 / orig_width as f64;
        let scale_y = max_height as f64 / orig_height as f64;
        let scale = scale_x.min(scale_y);

        let new_width = (orig_width as f64 * scale) as u32;
        let new_height = (orig_height as f64 * scale) as u32;

        self.resize(image, new_width, new_height, algorithm)
    }

    /// ブライトネス調整
    fn adjust_brightness(&self, image: &DynamicImage, brightness: f32) -> Result<DynamicImage> {
        let brightness = brightness.clamp(-1.0, 1.0);
        let adjustment = (brightness * 255.0) as i16;

        let rgb_image = image.to_rgb8();
        let mut new_data = Vec::with_capacity(rgb_image.len());

        for pixel in rgb_image.pixels() {
            let [r, g, b] = pixel.0;
            new_data.push(((r as i16 + adjustment).clamp(0, 255)) as u8);
            new_data.push(((g as i16 + adjustment).clamp(0, 255)) as u8);
            new_data.push(((b as i16 + adjustment).clamp(0, 255)) as u8);
        }

        let new_image = ImageBuffer::from_raw(image.width(), image.height(), new_data)
            .ok_or_else(|| MediaError::Image(
                image::ImageError::Parameter(
                    image::error::ParameterError::from_kind(
                        image::error::ParameterErrorKind::DimensionMismatch
                    )
                )
            ))?;

        Ok(DynamicImage::ImageRgb8(new_image))
    }

    /// コントラスト調整
    fn adjust_contrast(&self, image: &DynamicImage, contrast: f32) -> Result<DynamicImage> {
        let contrast = contrast.clamp(0.0, 2.0);
        let factor = contrast;

        let rgb_image = image.to_rgb8();
        let mut new_data = Vec::with_capacity(rgb_image.len());

        for pixel in rgb_image.pixels() {
            let [r, g, b] = pixel.0;
            new_data.push((((r as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0)) as u8);
            new_data.push((((g as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0)) as u8);
            new_data.push((((b as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0)) as u8);
        }

        let new_image = ImageBuffer::from_raw(image.width(), image.height(), new_data)
            .ok_or_else(|| MediaError::Image(
                image::ImageError::Parameter(
                    image::error::ParameterError::from_kind(
                        image::error::ParameterErrorKind::DimensionMismatch
                    )
                )
            ))?;

        Ok(DynamicImage::ImageRgb8(new_image))
    }

    /// 彩度調整
    fn adjust_saturation(&self, image: &DynamicImage, saturation: f32) -> Result<DynamicImage> {
        let saturation = saturation.clamp(0.0, 2.0);
        let rgb_image = image.to_rgb8();
        let mut new_data = Vec::with_capacity(rgb_image.len());

        for pixel in rgb_image.pixels() {
            let [r, g, b] = pixel.0;

            // RGBをHSVに変換して彩度を調整
            let (h, s, v) = rgb_to_hsv(r, g, b);
            let new_s = (s * saturation).min(1.0);
            let (new_r, new_g, new_b) = hsv_to_rgb(h, new_s, v);

            new_data.push(new_r);
            new_data.push(new_g);
            new_data.push(new_b);
        }

        let new_image = ImageBuffer::from_raw(image.width(), image.height(), new_data)
            .ok_or_else(|| MediaError::Image(
                image::ImageError::Parameter(
                    image::error::ParameterError::from_kind(
                        image::error::ParameterErrorKind::DimensionMismatch
                    )
                )
            ))?;

        Ok(DynamicImage::ImageRgb8(new_image))
    }

    /// ガンマ補正
    fn adjust_gamma(&self, image: &DynamicImage, gamma: f32) -> Result<DynamicImage> {
        let gamma = gamma.clamp(0.1, 3.0);
        let inv_gamma = 1.0 / gamma;

        let rgb_image = image.to_rgb8();
        let mut new_data = Vec::with_capacity(rgb_image.len());

        for pixel in rgb_image.pixels() {
            let [r, g, b] = pixel.0;
            new_data.push(((r as f32 / 255.0).powf(inv_gamma) * 255.0) as u8);
            new_data.push(((g as f32 / 255.0).powf(inv_gamma) * 255.0) as u8);
            new_data.push(((b as f32 / 255.0).powf(inv_gamma) * 255.0) as u8);
        }

        let new_image = ImageBuffer::from_raw(image.width(), image.height(), new_data)
            .ok_or_else(|| MediaError::Image(
                image::ImageError::Parameter(
                    image::error::ParameterError::from_kind(
                        image::error::ParameterErrorKind::DimensionMismatch
                    )
                )
            ))?;

        Ok(DynamicImage::ImageRgb8(new_image))
    }

    /// ガウシアンブラー
    fn gaussian_blur(&self, image: &DynamicImage, sigma: f32) -> Result<DynamicImage> {
        // 簡易的なブラー実装（実際にはより高度なアルゴリズムを使用する必要がある）
        Ok(image.blur(sigma))
    }

    /// シャープネス
    fn sharpen(&self, image: &DynamicImage, amount: f32) -> Result<DynamicImage> {
        // 簡易的なシャープネス実装（実際にはより高度なアルゴリズムを使用する必要がある）
        let amount = amount.clamp(0.0, 2.0);
        Ok(image.unsharpen(amount, 1))
    }
}

/// RGB to HSV 変換
fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max =r.max(g).max(b);
    let min =r.min(g).min(b);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max;

    (h, s, v)
}

/// HSV to RGB 変換
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r_prime + m) * 255.0) as u8;
    let g = ((g_prime + m) * 255.0) as u8;
    let b = ((b_prime + m) * 255.0) as u8;

    (r, g, b)
}