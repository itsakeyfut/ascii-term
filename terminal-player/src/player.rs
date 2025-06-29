use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver, Sender};
use tokio::time;

use media_core::{MediaFile, MediaType, video::VideoDecoder};
use crate::renderer::{AsciiRenderer, RenderConfig, RenderedFrame};

/// プレイヤー設定
pub struct PlayerConfig {
    pub fps: Option<f64>,
    pub loop_playback: bool,
    pub char_map_index: u8,
    pub grayscale: bool,
    pub width_modifier: u32,
    pub allow_frame_skip: bool,
    pub add_newlines: bool,
    pub enable_audio: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            fps: None,
            loop_playback: false,
            char_map_index: 0,
            grayscale: false,
            width_modifier: 1,
            allow_frame_skip: false,
            add_newlines: false,
            enable_audio: true,
        }
    }
}

/// プレイヤー制御コマンド
#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Play,
    Pause,
    Stop,
    Seek(Duration),
    SetVolume(f32),
    Mute,
    Unmute,
    TogglePlayPause,
    ToggleMute,
    SetCharMap(u8),
    ToggleGrayscale,
    Resize(u16, u16),
}

/// プレイヤーの状態
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerStatus {
    Playing,
    Paused,
    Stopped,
}

/// メディアプレーヤー
pub struct Player {
    media_file: MediaFile,
    config: PlayerConfig,
    state: Arc<AtomicBool>, // true = playing, false = paused
    stop_signal: Arc<AtomicBool>,

    // チャンネル
    command_tx: Sender<PlayerCommand>,
    command_rx: Receiver<PlayerCommand>,
    frame_tx: Sender<RenderedFrame>,
    frame_rx: Receiver<RenderedFrame>,

    // コンポーネント
    renderer: AsciiRenderer,
    // TODO: 追加が必要
    // terminal
    // audio_player 
}