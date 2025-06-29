use std::io::{stdout, Write};
use std::time::Duration;

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
};

use crate::player::PlayerCommand;
use crate::renderer::RenderedFrame;

/// ターミナル表示とユーザー入力を管理
pub struct Terminal {
    command_tx: Sender<PlayerCommand>,
    frame_rx: Receiver<RenderedFrame>,
    grayscale_mode: bool,
    last_frame: Option<RenderedFrame>,
}

impl Terminal {
    /// 新しいターミナルを作成
    pub fn new(
        command_tx: Sender<PlayerCommand>,
        frame_rx: Receiver<RenderedFrame>,
        grayscale_mode: bool,
    ) -> Result<Self> {
        Ok(Self {
            command_tx,
            frame_rx,
            grayscale_mode,
            last_frame: None,
        })
    }

    /// ターミナルを初期化
    fn init_terminal(&self) -> Result<()> {
        execute!(stdout(), EnterAlternateScreen, SetTitle("Ascii Term"))?;
        terminal::enable_raw_mode()?;
        self.cleanup_terminal()?;
        Ok(())
    }

    /// ターミナルの実行を開始
    fn cleanup_terminal(&self) -> Result<()> {
        execute!(
            stdout(),
            ResetColor,
            Clear(ClearType::All),
            Show,
            LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    /// 画面をクリア
    fn clear_screen(&self) -> Result<()> {
        execute!(
            stdout(),
            Clear(ClearType::All),
            Hide,
            MoveTo(0, 0),
        )?;
        stdout().flush()?;
        Ok(())
    }

    /// フレームを表示
    fn display_frame(&mut self, frame: &RenderedFrame) -> Result<()> {
        if self.grayscale_mode {
            self.display_grayscale_frame(frame)
        } else {
            self.display_colored_frame(frame)
        }
    }

    /// グレースケールフレームを表示
    fn display_grayscale_frame(&self, frame: &RenderedFrame) -> Result<()> {
        execute!(stdout(), MoveTo(0, 0), Print(&frame.ascii_text))?;
        stdout().flush()?;
        Ok(())
    }

    /// カラーフレームを表示
    fn display_colored_frame(&self, frame: &RenderedFrame) -> Result<()> {
        let mut colored_string = String::with_capacity(frame.ascii_text.len() * 20);
        let chars: Vec<char> = frame.ascii_text.chars().collect();

        for (i, ch) in chars.iter().enumerate() {
            // RGB色情報を取得
            let rgb_index = i * 3;
            if rgb_index + 2 < frame.rgb_data.len() {
                let r = frame.rgb_data[rgb_index];
                let g = frame.rgb_data[rgb_index + 1];
                let b = frame.rgb_data[rgb_index + 2];

                let color = Color::Rgb { r, g, b };
                colored_string.push_str(&format!("{}", ch.stylize().with(color)));
            } else {
                colored_string.push(*ch);
            }
        }

        execute!(stdout(), MoveTo(0, 0), Print(colored_string))?;
        stdout().flush()?;
        Ok(())
    }
}