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
}