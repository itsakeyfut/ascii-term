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

    /// ターミナルをクリーンアップ
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
    fn display_frame(&self, frame: &RenderedFrame) -> Result<()> {
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

    /// 入力イベントを処理
    fn handle_input_event(&mut self) -> Result<bool> {
        let event = event::read()?;

        match event {
            Event::Key(KeyEvent { code, modifiers, .. }) => {
                match (code, modifiers) {
                    // 終了
                    (KeyCode::Char('q'), _) |
                    (KeyCode::Char('Q'), _) |
                    (KeyCode::Esc, _) |
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        self.send_command(PlayerCommand::Stop)?;
                        return Ok(true);
                    }

                    // 再生/一時停止
                    (KeyCode::Char(' '), _) => {
                        self.send_command(PlayerCommand::TogglePlayPause)?;
                    }

                    // ミュートの切り替え
                    (KeyCode::Char('m'), _) | (KeyCode::Char('M'), _) => {
                        self.send_command(PlayerCommand::ToggleMute)?;
                    }

                    // グレースケール切り替え
                    (KeyCode::Char('g'), _) | (KeyCode::Char('G'), _) => {
                        self.grayscale_mode = !self.grayscale_mode;
                        self.send_command(PlayerCommand::ToggleGrayscale)?;

                        // 最後のフレームを再描画
                        if let Some(ref frame) = self.last_frame {
                            self.display_frame(frame)?;
                        }
                    }

                    // 文字マップ変更 (0-9)
                    (KeyCode::Char(digit), _) if digit.is_ascii_digit() => {
                        let index = digit.to_digit(10).unwrap_or(0) as u8;
                        self.send_command(PlayerCommand::SetCharMap(index))?;
                    }

                    // ヘルプ表示
                    (KeyCode::Char('h'), _) | (KeyCode::Char('H'), _) => {
                        self.show_help()?;
                    }

                    _ => {}
                }
            }

            Event::Resize(width, height) => {
                self.send_command(PlayerCommand::Resize(width, height))?;
                self.clear_screen()?;
            }

            _ => {}
        }

        Ok(false)
    }

    /// ヘルプを表示
    fn show_help(&self) -> Result<()> {
        let help_text = r#"
            tplay - Terminal Media Player

            Controls:
            Space       Play/Pause
            Q, Esc      Quit
            M           Mute/Unmute
            G           Toggle Grayscale
            0-9         Change character map
            H           Show this help

            Press any key to continue...
        "#;

        execute!(
            stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print(help_text)
        )?;
        stdout().flush()?;

        // キー入力を待つ
        event::read()?;
        
        // 画面をクリアして前の状態に戻る
        self.clear_screen()?;
        if let Some(ref frame) = self.last_frame {
            self.display_frame(frame)?;
        }

        Ok(())
    }

    /// コマンドを送信
    fn send_command(&self, command: PlayerCommand) -> Result<()> {
        self.command_tx
            .send(command)
            .map_err(|e| anyhow::anyhow!("Failed to send command: {}", e))?;
        Ok(())
    }
}