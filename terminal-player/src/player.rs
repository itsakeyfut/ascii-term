use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver, Sender};
use media_core::PipelineBuilder;
use tokio::time;

use media_core::{MediaFile, MediaType, video::VideoDecoder};
use crate::audio::AudioPlayer;
use crate::renderer::{AsciiRenderer, RenderConfig, RenderedFrame};
use crate::terminal::Terminal;

/// プレイヤー設定
#[derive(Debug, Clone)]
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
pub enum PlayerState {
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
            width_modifier: config.width_modifier,
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
        // ターミナルを初期化
        let terminal = Terminal::new(
            self.command_tx.clone(),
            self.frame_rx.clone(),
            self.config.grayscale,
        )?;
        self.terminal = Some(terminal);

        // メディアタイプに応じて適切な再生方法を選択
        match self.media_file.media_type {
            MediaType::Video => self.play_video().await,
            MediaType::Audio => self.play_audio().await,
            MediaType::Image => self.display_image().await,
            MediaType::Unknown => Err(anyhow::anyhow!("Unknown media type")),
        }
    }

    /// 動画再生 (Pipelineを使用)
    async fn play_video(&mut self) -> Result<()> {
        let fps = self.config.fps
            .or(self.media_file.info.fps)
            .unwrap_or(30.0);
        
        let frame_duration = Duration::from_secs_f64(1.0 / fps);

        let mut pipeline = PipelineBuilder::new()
            .buffer_size(5)
            .build();
        
        // メディアファイルをPipelineに設定
        pipeline.set_media(self.media_file.clone())?;
        pipeline.start()?;
        
        println!("Video pipeline started. Press 'space' to play/pause, 'q' to quit.");
        
        // ターミナルを別スレッドで開始
        if let Some(terminal) = self.terminal.take() {
            let _terminal_handle = tokio::spawn(async move {
                if let Err(e) = terminal.run().await {
                    eprintln!("Terminal error: {}", e);
                }
            });
        }

        // オーディオを開始
        if let Some(audio_player) = &mut self.audio_player {
            audio_player.play()?;
        }

        // 自動再生を開始
        self.state.store(true, Ordering::Relaxed);

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
                    // *** Pipeline から次のフレームを取得 ***
                    match pipeline.next_frame()? {
                        Some(video_frame) => {
                            // フレームをレンダリング
                            let rendered_frame = self.renderer.render_video_frame(&video_frame)?;
                            
                            // フレームを送信
                            if self.frame_tx.send(rendered_frame).is_err() {
                                break; // 受信側が終了
                            }
                            
                            frame_count += 1;
                            last_frame_time = now;
                            
                            // デバッグ情報
                            if frame_count % 30 == 0 {
                                println!("Frames processed: {}", frame_count);
                            }
                        }
                        None => {
                            // ストリーム終了
                            if pipeline.is_finished() {
                                if self.config.loop_playback {
                                    // ループ再生 (簡易実装)
                                    pipeline.stop()?;
                                    pipeline.set_media(self.media_file.clone())?;
                                    pipeline.start()?;
                                    println!("Looping video...");
                                } else {
                                    println!("Video finished.");
                                    break;
                                }
                            }
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

        // クリーンアップ
        pipeline.stop()?;
        if let Some(audio_player) = &mut self.audio_player {
            audio_player.stop()?;
        }

        println!("Video playback finished. Total frames: {}", frame_count);
        Ok(())
    }

    /// 音声再生
    async fn play_audio(&mut self) -> Result<()> {
        if let Some(audio_player) = &mut self.audio_player {
            audio_player.play()?;
        }
            
        // ターミナルを開始（音声再生制御用）
        if let Some(terminal) = self.terminal.take() {
            let _terminal_handle = tokio::spawn(async move {
                if let Err(e) = terminal.run().await {
                    eprintln!("Terminal error: {}", e);
                }
            });
        }

        // 制御ループ
        loop {
            if self.stop_signal.load(Ordering::Relaxed) {
                break;
            }

            while let Ok(command) = self.command_rx.try_recv() {
                self.handle_command(command).await?;
            }

            time::sleep(Duration::from_millis(100)).await;
        }

        if let Some(audio_player) = &mut self.audio_player {
            audio_player.stop()?;
        }

        Ok(())
    }

    /// 静止画表示
    async fn display_image(&mut self) -> Result<()> {
        // 画像を読み込み
        let image = image::open(&self.media_file.path)?;
        let rendered_frame = self.renderer.render_image(&image)?;
        
        // ターミナルを開始
        if let Some(terminal) = self.terminal.take() {
            let _terminal_handle = tokio::spawn(async move {
                if let Err(e) = terminal.run().await {
                    eprintln!("Terminal error: {}", e);
                }
            });
        }

        // フレームを送信
        self.frame_tx.send(rendered_frame)?;

        // 制御ループ
        loop {
            if self.stop_signal.load(Ordering::Relaxed) {
                break;
            }

            while let Ok(command) = self.command_rx.try_recv() {
                self.handle_command(command).await?;
            }

            time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// コマンドを処理
    async fn handle_command(&mut self, command: PlayerCommand) -> Result<()> {
        match command {
            PlayerCommand::Play => {
                self.state.store(true, Ordering::Relaxed);
                if let Some(audio_player) = &mut self.audio_player {
                    audio_player.resume()?;
                }
                println!("Playing...");
            }
            PlayerCommand::Pause => {
                self.state.store(false, Ordering::Relaxed);
                if let Some(audio_player) = &mut self.audio_player {
                    audio_player.pause()?;
                }
                println!("Paused.");
            }
            PlayerCommand::Stop => {
                self.stop_signal.store(true, Ordering::Relaxed);
                if let Some(audio_player) = &mut self.audio_player {
                    audio_player.stop()?;
                }
                println!("Stopped.");
            }
            PlayerCommand::TogglePlayPause => {
                let current_state = self.state.load(Ordering::Relaxed);
                if current_state {
                    Box::pin(self.handle_command(PlayerCommand::Pause)).await?;
                } else {
                    Box::pin(self.handle_command(PlayerCommand::Play)).await?;
                }
            }
            PlayerCommand::ToggleMute => {
                if let Some(audio_player) = &mut self.audio_player {
                    audio_player.toggle_mute()?;
                    println!("Mute toggled.");
                }
            }
            PlayerCommand::SetCharMap(index) => {
                self.renderer.set_char_map(index);
                println!("Character map changed to: {}", 
                    crate::char_maps::get_char_map_name(index));
            }
            PlayerCommand::ToggleGrayscale => {
                self.config.grayscale = !self.config.grayscale;
                self.renderer.set_grayscale(self.config.grayscale);
                println!("Grayscale mode: {}", self.config.grayscale);
            }
            PlayerCommand::Resize(width, height) => {
                self.renderer.update_resolution(width, height);
                println!("Resolution updated: {}x{}", width, height);
            }
            _ => {
                // その他のコマンドは後で実装
            }
        }
        Ok(())
    }

    /// コマンド送信用のハンドルを取得
    pub fn command_sender(&self) -> Sender<PlayerCommand> {
        self.command_tx.clone()
    }
}