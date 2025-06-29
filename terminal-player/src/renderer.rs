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
}