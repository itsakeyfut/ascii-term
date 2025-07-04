use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver, Sender};
use tokio::time;

use media_core::{MediaFile, MediaType, video::VideoDecoder};
use crate::audio::AudioPlayer;
use crate::renderer::{AsciiRenderer, RenderConfig, RenderedFrame};
use crate::terminal::Terminal;

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
    terminal: Option<Terminal>,
    audio_player: Option<AudioPlayer>,
}

impl Player {
    /// 新しいプレイヤーを作成
    pub fn new(media_file: MediaFile, config: PlayerConfig) -> Result<Self> {
        let (command_tx, command_rx) = unbounded();
        let (frame_tx, frame_rx) = unbounded();

        let render_config = RenderConfig {
            target_width: 80,
            target_height: 24,
            char_map_index: config.char_map_index,
            grayscale: config.grayscale,
            add_newlines: config.add_newlines,
            width_modifiers: config.width_modifier,
        };

        let renderer = AsciiRenderer::new(render_config);

        // オーディオプレイヤーの初期化
        let audio_player = if config.enable_audio && media_file.info.has_audio {
            Some(AudioPlayer::new(&media_file.path)?)
        } else {
            None
        };

        Ok(Self {
            media_file,
            config,
            state: Arc::new(AtomicBool::new(false)),
            stop_signal: Arc::new(AtomicBool::new(false)),
            command_tx,
            command_rx,
            frame_tx,
            frame_rx,
            renderer,
            terminal: None,
            audio_player,
        })
    }
}