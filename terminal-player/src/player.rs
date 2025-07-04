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

    /// プレイヤーを実行
    pub async fn run(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// 動画再生
    async fn play_video(&mut self) -> Result<()> {
        let fps = self.config.fps
            .or(self.media_file.info.fps)
            .unwrap_or(30.0);

        let frame_duration = Duration::from_secs_f64(1.0 / fps);

        // ビデオコーダーを作成
        let mut decoder = VideoDecoder::new(&self.media_file)?;

        // ターミナルを別スレッドで開始
        if let Some(terminal) = self.terminal.take() {
            let terminal_handle = tokio::spawn(async move {
                terminal.run().await
            });
        }

        // オーディオを開始
        if let Some(audio_player) = &mut self.audio_player {
            audio_player.play()?;
        }

        // フレーム送信ループ
        let mut last_frame_time = Instant::now();
        let mut frame_count = 0u64;

        loop {
            // 停止シグナルをチェック
            if self.stop_signal.load(Ordering::Relaxed) {
                break;
            }

            // コマンドを処理
            while let Ok(command) = self.command_rx.try_recv() {
                self.handle_command(command).await?;
            }

            // 再生中の場合のみフレームを処理
            if self.state.load(Ordering::Relaxed) {
                let now = Instant::now();
                let elapsed = now.duration_since(last_frame_time);
                
                if elapsed >= frame_duration {
                    // フレームスキップの計算
                    let frames_to_skip = if self.config.allow_frame_skip && elapsed > frame_duration * 2 {
                        (elapsed.as_secs_f64() / frame_duration.as_secs_f64()) as usize - 1
                    } else {
                        0
                    };

                    // パケットを読み込んでデコード
                    if let Ok((stream, packet)) = self.media_file.format_context().read_packet() {
                        if let Some(video_frame) = decoder.decode_next_frame(&packet)? {
                            // フレームをレンダリング
                            let rendered_frame = self.renderer.render_video_frame(&video_frame)?;
                            
                            // フレームを送信
                            if self.frame_tx.send(rendered_frame).is_err() {
                                break; // 受信側が終了
                            }
                            
                            frame_count += 1;
                            last_frame_time = now;
                            
                            // フレームスキップ
                            for _ in 0..frames_to_skip {
                                if let Ok((_, packet)) = self.media_file.format_context().read_packet() {
                                    let _ = decoder.decode_next_frame(&packet);
                                }
                            }
                        }
                    } else {
                        // ストリーム終了
                        if self.config.loop_playback {
                            // ループ再生
                            self.seek_to_start().await?;
                        } else {
                            break;
                        }
                    }
                } else {
                    // フレームタイミングまで待機
                    time::sleep(Duration::from_millis(1)).await;
                }
            } else {
                // 一時停止中
                time::sleep(Duration::from_millis(16)).await;
            }
        }

        // オーディオを停止
        if let Some(audio_player) = &mut self.audio_player {
            audio_player.stop()?;
        }

        Ok(())
    }

    /// コマンドを処理
    async fn handle_command(&mut self, command: PlayerCommand) -> Result<()> {
        match command {
            PlayerComamnd::Play => {
                self.state.store(true, Ordering::Relaxed);
                if let Some(audio_player) = &mut self.audio_player {
                    audio_player.resume()?;
                }
            }
            PlayerCommand::Pause => {
                self.state.store(false, Ordering::Relaxed);
                if let Some(audio_player) = &mut self.audio_player {
                    audio_player.pause()?;
                }
            }
            PlayerCommand::Stop => {
                self.stop_signal.store(true, Ordering::Relaxed);
                if let Some(audio_player) = &mut self.audio_player {
                    audio_player.stop()?;
                }
            }
            PlayerCommand::TogglePlayPause => {
                let current_state = self.state.load(Ordering::Relaxed);
                if current_state {
                    self.handle_command(PlayerCommand::Pause).await?;
                } else {
                    self.handle_command(PlayerCommand::Play).await?;
                }
            }
            PlayerCommand::ToggleMute => {
                if let Some(audio_player) = &mut self.audio_player {
                    audio_player.toggle_mute()?;
                }
            }
            PlayerCommand::SetCharMap(index) => {
                self.renderer.set_char_map(index);
            }
            PlayerCommand::ToggleGrayscale => {
                self.config.grayscale = !self.config.grayscale;
                self.renderer.set_grayscale(self.config.grayscale);
            }
            PlayerCommand::Resize(width, height) => {
                self.renderer.update_resolution(width, height);
            }
            _ => {
                // その他のコマンドは後で実装する
            }
        }
        Ok(())
    }
}