//! ターミナルのライフサイクルとユーザー入力処理

use std::io::{Write, stdout};
use std::time::Duration;

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Print, ResetColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
};

use crate::player::PlayerCommand;
use crate::renderer::RenderedFrame;

mod output;

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

    /// ターミナルの実行を開始
    pub async fn run(mut self) -> Result<()> {
        // ターミナルの初期化
        self.init_terminal()?;

        // メインループ
        loop {
            // イベントをポーリング
            if event::poll(Duration::from_millis(16))? && self.handle_input_event()? {
                break; // 終了
            }

            // フレームの受信と描画
            if let Ok(frame) = self.frame_rx.try_recv() {
                self.display_frame(&frame)?;
                self.last_frame = Some(frame);
            }
        }

        // クリーンアップ
        self.cleanup_terminal()?;
        Ok(())
    }

    /// ターミナルを初期化
    fn init_terminal(&self) -> Result<()> {
        execute!(
            stdout(),
            EnterAlternateScreen,
            SetTitle("ascii-term - Ascii Rendered Media Player")
        )?;
        terminal::enable_raw_mode()?;
        self.clear_screen()?;
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
        execute!(stdout(), Clear(ClearType::All), Hide, MoveTo(0, 0),)?;
        stdout().flush()?;
        Ok(())
    }

    /// 入力イベントを処理
    fn handle_input_event(&mut self) -> Result<bool> {
        let event = event::read()?;

        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => {
                match (code, modifiers) {
                    // 終了
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Char('Q'), _)
                    | (KeyCode::Esc, _)
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        self.send_command(PlayerCommand::Stop)?;
                        return Ok(true);
                    }

                    // 再生/一時停止
                    (KeyCode::Char(' '), _) => {
                        self.send_command(PlayerCommand::TogglePlayPause)?;
                    }

                    // ミュート切り替え
                    (KeyCode::Char('m'), _) | (KeyCode::Char('M'), _) => {
                        self.send_command(PlayerCommand::ToggleMute)?;
                    }

                    // グレースケール切り替え
                    (KeyCode::Char('g'), _) | (KeyCode::Char('G'), _) => {
                        self.grayscale_mode = !self.grayscale_mode;
                        self.send_command(PlayerCommand::ToggleGrayscale)?;

                        // 最後のフレームを再描画
                        if let Some(frame) = self.last_frame.take() {
                            self.display_frame(&frame)?;
                        }
                    }

                    // 文字マップ変更（0-9）
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

            Event::Resize(_, _) => {
                // 解像度は起動時に固定。画面クリアして最終フレームを再描画するだけ
                self.clear_screen()?;
                if let Some(ref frame) = self.last_frame.clone() {
                    self.display_frame(frame)?;
                }
            }

            _ => {}
        }

        Ok(false)
    }

    /// ヘルプを表示
    fn show_help(&mut self) -> Result<()> {
        let help_text = r#"
            ascii-term - Ascii Rendered Media Player

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
        if let Some(frame) = self.last_frame.take() {
            self.display_frame(&frame)?;
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
