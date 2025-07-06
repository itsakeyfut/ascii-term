use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, unbounded};
use media_core::PipelineBuilder;
use tokio::time;

use crate::audio::AudioPlayer;
use crate::renderer::{AsciiRenderer, RenderConfig, RenderedFrame};
use crate::terminal::Terminal;
use media_core::{MediaFile, MediaType};

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

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    Playing,
    Paused,
    Stopped,
}

pub struct Player {
    media_file: MediaFile,
    config: PlayerConfig,
    state: Arc<AtomicBool>, // true = playing, false = paused
    stop_signal: Arc<AtomicBool>,

    // Channel
    command_tx: Sender<PlayerCommand>,
    command_rx: Receiver<PlayerCommand>,
    frame_tx: Sender<RenderedFrame>,
    frame_rx: Receiver<RenderedFrame>,

    // Component
    renderer: AsciiRenderer,
    terminal: Option<Terminal>,
    audio_player: Option<AudioPlayer>,
}

impl Player {
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

        let audio_player = if config.enable_audio && media_file.info.has_audio {
            match AudioPlayer::new(&media_file.path) {
                Ok(player) => {
                    println!("Audio player initialized successfully");
                    Some(player)
                }
                Err(e) => {
                    eprintln!("Warning: Audio initialization failed: {}", e);
                    eprintln!("Continuing with video-only playback...");
                    None
                }
            }
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

    pub async fn run(&mut self) -> Result<()> {
        let terminal = Terminal::new(
            self.command_tx.clone(),
            self.frame_rx.clone(),
            self.config.grayscale,
        )?;
        self.terminal = Some(terminal);

        match self.media_file.media_type {
            MediaType::Video => self.play_video().await,
            MediaType::Audio => self.play_audio().await,
            MediaType::Image => self.display_image().await,
            MediaType::Unknown => Err(anyhow::anyhow!("Unknown media type")),
        }
    }

    async fn play_video(&mut self) -> Result<()> {
        let fps = self.config.fps.or(self.media_file.info.fps).unwrap_or(30.0);

        let frame_duration = Duration::from_secs_f64(1.0 / fps);

        let video_media_file = MediaFile::open(&self.media_file.path)?;
        let mut pipeline = PipelineBuilder::new().buffer_size(8).build();

        pipeline.set_media(video_media_file)?;
        pipeline.start()?;

        println!("Video pipeline started. Press 'space' to play/pause, 'q' to quit.");

        if let Some(terminal) = self.terminal.take() {
            let _terminal_handle = tokio::spawn(async move {
                if let Err(e) = terminal.run().await {
                    eprintln!("Terminal error: {}", e);
                }
            });
        }

        self.state.store(true, Ordering::Relaxed);

        // 音声と動画を同期開始
        let audio_started = if let Some(audio_player) = &mut self.audio_player {
            match audio_player.play() {
                Ok(_) => {
                    println!("Audio started successfully with video");
                    true
                }
                Err(e) => {
                    eprintln!("Warning: Failed to start audio: {}", e);
                    false
                }
            }
        } else {
            false
        };

        let mut last_frame_time = Instant::now();
        let mut frame_count = 0u64;
        let playback_start_time = Instant::now();
        let mut video_finished = false;

        loop {
            if self.stop_signal.load(Ordering::Relaxed) {
                println!("Stop signal received, exiting");
                break;
            }

            while let Ok(command) = self.command_rx.try_recv() {
                self.handle_command(command).await?;
            }

            if self.state.load(Ordering::Relaxed) && !video_finished {
                let now = Instant::now();
                let elapsed = now.duration_since(last_frame_time);

                if elapsed >= frame_duration || self.config.allow_frame_skip {
                    match pipeline.next_frame()? {
                        Some(video_frame) => {
                            let rendered_frame = self.renderer.render_video_frame(&video_frame)?;

                            if self.frame_tx.send(rendered_frame).is_err() {
                                println!("Frame receiver closed");
                                break;
                            }

                            frame_count += 1;
                            last_frame_time = now;

                            if self.config.allow_frame_skip && elapsed > frame_duration * 2 {
                                println!("Frame skip detected at frame {}", frame_count);
                            }

                            // if frame_count % 900 == 0 {
                            //     let playback_time = playback_start_time.elapsed().as_secs_f64();
                            //     let expected_time = frame_count as f64 / fps;
                            //     println!(
                            //         "Video frames: {}, playback: {:.1}s, expected: {:.1}s",
                            //         frame_count, playback_time, expected_time
                            //     );
                            // }
                        }
                        None => {
                            if pipeline.is_finished() {
                                println!("Video stream finished");
                                video_finished = true;

                                if self.config.loop_playback {
                                    println!("Restarting video loop...");
                                    pipeline.stop()?;

                                    let loop_media_file = MediaFile::open(&self.media_file.path)?;
                                    pipeline.set_media(loop_media_file)?;
                                    pipeline.start()?;

                                    video_finished = false;
                                    println!("Video loop restarted");
                                } else {
                                    println!("Video finished, waiting for audio to complete...");
                                    break;
                                }
                            } else {
                                time::sleep(Duration::from_millis(10)).await;
                            }
                        }
                    }
                } else {
                    time::sleep(Duration::from_millis(1)).await;
                }
            } else {
                if video_finished && audio_started {
                    if let Some(audio_player) = &self.audio_player {
                        if audio_player.is_playing() {
                            println!("Waiting for audio to finish...");
                            time::sleep(Duration::from_millis(1000)).await;
                            continue;
                        } else {
                            println!("Audio finished");
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    time::sleep(Duration::from_millis(16)).await;
                }
            }
        }

        if audio_started && !self.config.loop_playback {
            println!("Ensuring audio completion...");
            let audio_wait_start = Instant::now();
            const MAX_AUDIO_WAIT: Duration = Duration::from_secs(60);

            while audio_wait_start.elapsed() < MAX_AUDIO_WAIT {
                if let Some(audio_player) = &self.audio_player {
                    if !audio_player.is_playing() {
                        println!("Audio playback completed");
                        break;
                    }
                } else {
                    break;
                }

                if self.stop_signal.load(Ordering::Relaxed) {
                    println!("Stop signal received during audio wait");
                    break;
                }

                while let Ok(command) = self.command_rx.try_recv() {
                    self.handle_command(command).await?;
                }

                time::sleep(Duration::from_millis(500)).await;
            }

            if audio_wait_start.elapsed() >= MAX_AUDIO_WAIT {
                println!("Audio wait timeout reached");
            }
        }

        println!("Cleaning up video and audio resources...");
        pipeline.stop()?;

        if let Some(audio_player) = &mut self.audio_player {
            if let Err(e) = audio_player.stop() {
                eprintln!("Warning: Failed to stop audio: {}", e);
            } else {
                println!("Audio stopped successfully");
            }
        }

        let total_playback_time = playback_start_time.elapsed().as_secs_f64();
        let expected_time = frame_count as f64 / fps;
        println!(
            "Video playback finished. Total frames: {}, playback: {:.1}s, expected: {:.1}s",
            frame_count, total_playback_time, expected_time
        );
        Ok(())
    }

    async fn play_audio(&mut self) -> Result<()> {
        println!("Starting audio-only playback");

        if let Some(audio_player) = &mut self.audio_player {
            if let Err(e) = audio_player.play() {
                eprintln!("Warning: Failed to start audio: {}", e);
                return Err(anyhow::anyhow!("Failed to start audio playback"));
            }
            println!("Audio playback started");
        } else {
            return Err(anyhow::anyhow!("No audio player available"));
        }

        if let Some(terminal) = self.terminal.take() {
            let _terminal_handle = tokio::spawn(async move {
                if let Err(e) = terminal.run().await {
                    eprintln!("Terminal error: {}", e);
                }
            });
        }

        let playback_start = Instant::now();

        loop {
            if self.stop_signal.load(Ordering::Relaxed) {
                println!("Stop signal received");
                break;
            }

            while let Ok(command) = self.command_rx.try_recv() {
                self.handle_command(command).await?;
            }

            if let Some(audio_player) = &self.audio_player {
                if !audio_player.is_playing() {
                    println!("Audio playback completed naturally");
                    break;
                }
            } else {
                println!("Audio player unavailable");
                break;
            }

            time::sleep(Duration::from_millis(500)).await;
        }

        if let Some(audio_player) = &mut self.audio_player {
            if let Err(e) = audio_player.stop() {
                eprintln!("Warning: Failed to stop audio: {}", e);
            } else {
                println!("Audio stopped successfully");
            }
        }

        let total_time = playback_start.elapsed().as_secs_f64();
        println!("Audio playback finished. Total time: {:.1}s", total_time);
        Ok(())
    }

    async fn display_image(&mut self) -> Result<()> {
        let image = image::open(&self.media_file.path)?;
        let rendered_frame = self.renderer.render_image(&image)?;

        if let Some(terminal) = self.terminal.take() {
            let _terminal_handle = tokio::spawn(async move {
                if let Err(e) = terminal.run().await {
                    eprintln!("Terminal error: {}", e);
                }
            });
        }

        self.frame_tx.send(rendered_frame)?;

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

    async fn handle_command(&mut self, command: PlayerCommand) -> Result<()> {
        match command {
            PlayerCommand::Play => {
                println!("Play command received");
                self.state.store(true, Ordering::Relaxed);
                if let Some(audio_player) = &mut self.audio_player {
                    if let Err(e) = audio_player.resume() {
                        eprintln!("Warning: Failed to resume audio: {}", e);
                    } else {
                        println!("Audio resumed successfully");
                    }
                }
            }
            PlayerCommand::Pause => {
                println!("Pause command received");
                self.state.store(false, Ordering::Relaxed);
                if let Some(audio_player) = &mut self.audio_player {
                    if let Err(e) = audio_player.pause() {
                        eprintln!("Warning: Failed to pause audio: {}", e);
                    } else {
                        println!("Audio paused successfully");
                    }
                }
            }
            PlayerCommand::Stop => {
                println!("Stop command received");
                self.stop_signal.store(true, Ordering::Relaxed);
                if let Some(audio_player) = &mut self.audio_player {
                    if let Err(e) = audio_player.stop() {
                        eprintln!("Warning: Failed to stop audio: {}", e);
                    } else {
                        println!("Audio stopped successfully");
                    }
                }
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
                    if let Err(e) = audio_player.toggle_mute() {
                        eprintln!("Warning: Failed to toggle mute: {}", e);
                    } else {
                        let muted = audio_player.is_muted();
                        println!("Audio mute toggled: {}", if muted { "ON" } else { "OFF" });
                    }
                } else {
                    println!("Audio not available for mute toggle");
                }
            }
            PlayerCommand::SetVolume(volume) => {
                if let Some(audio_player) = &mut self.audio_player {
                    if let Err(e) = audio_player.set_volume(volume) {
                        eprintln!("Warning: Failed to set volume: {}", e);
                    } else {
                        println!("Volume set to: {:.2}", volume);
                    }
                } else {
                    println!("Audio not available for volume control");
                }
            }
            PlayerCommand::SetCharMap(index) => {
                self.renderer.set_char_map(index);
                println!(
                    "Character map changed to: {}",
                    crate::char_maps::get_char_map_name(index)
                );
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
                println!("Command not implemented: {:?}", command);
            }
        }
        Ok(())
    }

    pub fn command_sender(&self) -> Sender<PlayerCommand> {
        self.command_tx.clone()
    }
}
