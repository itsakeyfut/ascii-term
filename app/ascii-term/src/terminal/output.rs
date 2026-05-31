//! ターミナルへのフレーム描画（ANSI エンコード）

use std::io::{Write, stdout};

use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Stylize},
};

use crate::renderer::RenderedFrame;

impl super::Terminal {
    /// フレームを表示
    pub(super) fn display_frame(&mut self, frame: &RenderedFrame) -> Result<()> {
        if self.grayscale_mode {
            self.display_grayscale_frame(frame)
        } else {
            self.display_colored_frame(frame)
        }
    }

    /// グレースケールフレームを表示
    fn display_grayscale_frame(&self, frame: &RenderedFrame) -> Result<()> {
        let chars: Vec<char> = frame.ascii_text.chars().collect();
        let width = frame.width as usize;
        let height = frame.height as usize;
        let mut out = stdout();

        for y in 0..height {
            let row_start = y * width;
            let row_end = (row_start + width).min(chars.len());
            let row: String = chars[row_start..row_end].iter().collect();
            execute!(out, MoveTo(0, y as u16))?;
            write!(out, "{}", row)?;
        }

        out.flush()?;
        Ok(())
    }

    /// カラーフレームを表示
    fn display_colored_frame(&self, frame: &RenderedFrame) -> Result<()> {
        let chars: Vec<char> = frame.ascii_text.chars().collect();
        let width = frame.width as usize;
        let height = frame.height as usize;
        let mut out = stdout();

        for y in 0..height {
            let row_start = y * width;
            let row_end = (row_start + width).min(chars.len());

            execute!(out, MoveTo(0, y as u16))?;

            let mut row_string = String::with_capacity(width * 20);
            for (j, ch) in chars[row_start..row_end].iter().enumerate() {
                let rgb_index = (row_start + j) * 3;
                if rgb_index + 2 < frame.rgb_data.len() {
                    let r = frame.rgb_data[rgb_index];
                    let g = frame.rgb_data[rgb_index + 1];
                    let b = frame.rgb_data[rgb_index + 2];
                    let color = Color::Rgb { r, g, b };
                    row_string.push_str(&format!("{}", ch.stylize().with(color)));
                } else {
                    row_string.push(*ch);
                }
            }
            write!(out, "{}", row_string)?;
        }

        out.flush()?;
        Ok(())
    }
}
