use anyhow::Result;
use fast_image_resize as fr;
use image::{DynamicImage, ImageBuffer};

use crate::char_maps;
use media_core::video::VideoFrame;

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub target_width: u32,
    pub target_height: u32,
    pub char_map_index: u8,
    pub grayscale: bool,
    pub add_newlines: bool,
    pub width_modifier: u32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            target_width: 80,
            target_height: 24,
            char_map_index: 0,
            grayscale: false,
            add_newlines: false,
            width_modifier: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderedFrame {
    pub ascii_text: String,
    pub rgb_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct AsciiRenderer {
    config: RenderConfig,
    resizer: fr::Resizer,
}

impl AsciiRenderer {
    pub fn new(config: RenderConfig) -> Self {
        Self {
            config,
            resizer: fr::Resizer::new(),
        }
    }

    pub fn update_config(&mut self, config: RenderConfig) {
        self.config = config;
    }

    pub fn update_resolution(&mut self, width: u16, height: u16) {
        self.config.target_width = (width / self.config.width_modifier as u16) as u32;
        self.config.target_height = height as u32;
    }

    pub fn set_char_map(&mut self, index: u8) {
        self.config.char_map_index = index;
    }

    pub fn set_grayscale(&mut self, grayscale: bool) {
        self.config.grayscale = grayscale;
    }

    pub fn render_video_frame(&mut self, frame: &VideoFrame) -> Result<RenderedFrame> {
        let dynamic_image = frame
            .to_dynamic_image()
            .map_err(|e| anyhow::anyhow!("Failed to convert frame to image: {}", e))?;

        self.render_image(&dynamic_image)
    }

    pub fn render_image(&mut self, image: &DynamicImage) -> Result<RenderedFrame> {
        let resized_image = self.resize_image(image)?;

        let rgb_image = resized_image.to_rgb8();

        let (ascii_text, rgb_data) = self.image_to_ascii_with_color(&rgb_image);

        Ok(RenderedFrame {
            ascii_text,
            rgb_data,
            width: self.config.target_width,
            height: self.config.target_height,
        })
    }

    fn resize_image(&mut self, image: &DynamicImage) -> Result<DynamicImage> {
        let src_width = image.width();
        let src_height = image.height();

        if src_width == self.config.target_width && src_height == self.config.target_height {
            return Ok(image.clone());
        }

        let rgb_image = image.to_rgb8();

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
            &fr::ResizeOptions::new()
                .resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3)),
        )?;

        let resized_data = dst_image.into_vec();
        let resized_buffer = ImageBuffer::from_raw(
            self.config.target_width,
            self.config.target_height,
            resized_data,
        )
        .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;

        Ok(DynamicImage::ImageRgb8(resized_buffer))
    }

    fn image_to_ascii_with_color(
        &self,
        rgb_image: &ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    ) -> (String, Vec<u8>) {
        let char_map = char_maps::get_char_map(self.config.char_map_index);
        let (width, height) = rgb_image.dimensions();

        let mut ascii_text = String::with_capacity((width * height) as usize + height as usize);
        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);

        for y in 0..height {
            for x in 0..width {
                let pixel = rgb_image.get_pixel(x, y);
                let [r, g, b] = pixel.0;

                // ITU-R BT.709
                let luminance = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) as u8;

                let ch = char_maps::luminance_to_char(luminance, char_map);
                ascii_text.push(ch);

                rgb_data.push(r);
                rgb_data.push(g);
                rgb_data.push(b);
            }

            // Optional
            if self.config.add_newlines && y < height - 1 {
                ascii_text.push('\r');
                ascii_text.push('\n');

                // Add RGB data for new line characters (fill with black)
                rgb_data.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
            }
        }

        (ascii_text, rgb_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn test_ascii_renderer_creation() {
        let config = RenderConfig::default();
        let renderer = AsciiRenderer::new(config);
        assert_eq!(renderer.config.target_width, 80);
        assert_eq!(renderer.config.target_height, 24);
    }

    #[test]
    fn test_render_small_image() {
        let mut config = RenderConfig::default();
        config.target_width = 4;
        config.target_height = 2;

        let mut renderer = AsciiRenderer::new(config);

        // Create a small 2x2 image
        let mut img = RgbImage::new(2, 2);
        img.put_pixel(0, 0, Rgb([255, 255, 255])); // White
        img.put_pixel(1, 0, Rgb([0, 0, 0])); // Black
        img.put_pixel(0, 1, Rgb([128, 128, 128])); // Gray
        img.put_pixel(1, 1, Rgb([200, 200, 200])); // Light gray

        let dynamic_img = DynamicImage::ImageRgb8(img);
        let result = renderer.render_image(&dynamic_img).unwrap();

        assert_eq!(result.width, 4);
        assert_eq!(result.height, 2);
        assert!(!result.ascii_text.is_empty());
        assert_eq!(result.rgb_data.len(), 4 * 2 * 3); // width * height * RGB
    }
}
