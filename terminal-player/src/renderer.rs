use image::{DynamicImage, ImageBuffer};
use fast_image_resize as fr;
use anyhow::Result;

use media_core::video::VideoFrame;
use crate::char_maps;

/// レンダリング設定
#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub target_width: u32,
    pub target_height: u32,
    pub char_map_index: u8,
    pub grayscale: bool,
    pub add_newlines: bool,
    pub width_modifiers: u32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            target_width: 80,
            target_height: 24,
            char_map_index: 0,
            grayscale: false,
            add_newlines: false,
            width_modifiers: 1,
        }
    }
}

/// ASCII文字情報とRGB色情報を含む構造体
#[derive(Debug, Clone)]
pub struct RenderedFrame {
    pub ascii_text: String,
    pub rgb_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// ASCII アートレンダラー
pub struct AsciiRenderer {
    config: RenderConfig,
    resizer: fr::Resizer,
}

impl AsciiRenderer {
    /// 新しいレンダラーを作成
    pub fn new(config: RenderConfig) -> Self {
        Self {
            config,
            resizer: fr::Resizer::new(),
        }
    }

    /// 設定を更新
    pub fn update_config(&mut self, config: RenderConfig) {
        self.config = config;
    }

    /// ターミナルサイズに基づいて解像度を更新
    pub fn update_resolution(&mut self, width: u16, height: u16) {
        self.config.target_width = (width / self.config.width_modifiers as u16) as u32;
        self.config.target_height = height as u32;
    }

    /// 文字マップを変更
    pub fn set_char_map(&mut self, index: u8) {
        self.config.char_map_index = index;
    }

    /// グレースケールモードを切り替え
    pub fn set_grayscale(&mut self, grayscale: bool) {
        self.config.grayscale = grayscale;
    }

    /// 画像をリサイズ
    fn resize_image(&mut self, image: &DynamicImage) -> Result<DynamicImage> {
        let src_width = image.width();
        let src_height = image.height();

        if src_width == self.config.target_width && src_height == self.config.target_height {
            return Ok(image.clone());
        }

        // RGB画像に変換
        let rgb_image = image.to_rgb8();

        // リサイズ
        let src_image = fr::images::Image::from_vec_u8(
            src_width,
            src_height,
            rgb_image.into_raw(),
            fr::PixelType::U8x3,
        )?;

        let mut dst_image = fr::images::Image::new(
            self.config.target_width,
            self.config.target_height,
            fr::PixelType::U8x3,
        );

        self.resizer.resize(
            &src_image,
            &mut dst_image,
            &fr::ResizeOptions::new().resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3)),
        )?;

        let resized_data = dst_image.into_vec();
        let resized_buffer = ImageBuffer::from_raw(
            self.config.target_width,
            self.config.target_height,
            resized_data,
        ).ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;

        Ok(DynamicImage::ImageRgb8(resized_buffer))
    }
}