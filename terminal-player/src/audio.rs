use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use rodio::{OutputStream, Sink, Source};
use crossbeam_channel::{unbounded, Receiver, Sender};

use media_core::{MediaFile, audio::AudioDecoder};

/// 音声データソース（FFmpegからのデータ用）
struct AudioSource {
    receiver: Receiver<Vec<f32>>,
    sample_rate: u32,
    channels: u16,
    current_data: Vec<f32>,
    position: usize,
}

impl AudioSource {
    fn new(receiver: Receiver<Vec<f32>>, sample_rate: u32, channels: u16) -> Self {
        Self {
            receiver,
            sample_rate,
            channels,
            current_data: Vec::new(),
            position: 0,
        }
    }
}

impl Source for AudioSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Iterator for AudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // 現在のデータが空の場合、新しいデータを取得
        if self.position >= self.current_data.len() {
            match self.receiver.try_recv() {
                Ok(data) => {
                    self.current_data = data;
                    self.position = 0;
                }
                Err(_) => {
                    // データが無い場合は無音を返す
                    return Some(0.0);
                }
            }
        }

        // データを返す
        if self.position < self.current_data.len() {
            let sample = self.current_data[self.position];
            self.position += 1;
            Some(sample)
        } else {
            Some(0.0)
        }
    }
}

/// オーディオプレイヤー
pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
    is_muted: Arc<AtomicBool>,
    original_volume: f32,
    audio_sender: Option<Sender<Vec<f32>>>,
    decoder_thread: Option<thread::JoinHandle<()>>,
    stop_signal: Arc<AtomicBool>,
}

impl AudioPlayer {
    /// 新しいオーディオプレイヤーを作成
    pub fn new(file_path: &str) -> Result<Self> {
        println!("Initializing FFmpeg-based audio player for: {}", file_path);
        
        // オーディオストリームを初期化
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| {
                eprintln!("Failed to initialize audio stream: {}", e);
                anyhow::anyhow!("Failed to initialize audio stream: {}", e)
            })?;

        println!("Audio stream initialized successfully");

        // Sink を作成
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| {
                eprintln!("Failed to create audio sink: {}", e);
                anyhow::anyhow!("Failed to create audio sink: {}", e)
            })?;

        println!("Audio sink created successfully");

        // MediaFileを開く
        let media_file = MediaFile::open(file_path)
            .map_err(|e| {
                eprintln!("Failed to open media file: {}", e);
                anyhow::anyhow!("Failed to open media file: {}", e)
            })?;

        if !media_file.info.has_audio {
            return Err(anyhow::anyhow!("Media file has no audio stream"));
        }

        // 音声デコーダーを作成
        let mut audio_decoder = AudioDecoder::new(&media_file)
            .map_err(|e| {
                eprintln!("Failed to create audio decoder: {}", e);
                anyhow::anyhow!("Failed to create audio decoder: {}", e)
            })?;

        println!("Audio decoder created successfully");

        // 音声情報を取得
        let sample_rate = audio_decoder.sample_rate();
        let channels = audio_decoder.channels();
        
        println!("Audio format: {} channels, {} Hz", channels, sample_rate);

        // チャンネルとストップシグナルを作成
        let (audio_sender, audio_receiver) = unbounded();
        let stop_signal = Arc::new(AtomicBool::new(false));

        // 音声ソースを作成
        let audio_source = AudioSource::new(audio_receiver, sample_rate, channels);
        
        // 音声ソースをシンクに追加
        sink.append(audio_source);
        sink.pause(); // 最初は一時停止状態

        // デコーダースレッドを開始
        let decoder_stop_signal = stop_signal.clone();
        let decoder_sender = audio_sender.clone();
        let decoder_thread = thread::spawn(move || {
            decode_audio_loop(media_file, audio_decoder, decoder_sender, decoder_stop_signal);
        });

        println!("Audio player initialized successfully");

        Ok(Self {
            _stream,
            sink,
            is_muted: Arc::new(AtomicBool::new(false)),
            original_volume: 0.6,
            audio_sender: Some(audio_sender),
            decoder_thread: Some(decoder_thread),
            stop_signal,
        })
    }

    /// 再生を開始
    pub fn play(&mut self) -> Result<()> {
        println!("Starting audio playback");
        self.sink.play();
        Ok(())
    }

    /// 再生を一時停止
    pub fn pause(&mut self) -> Result<()> {
        println!("Pausing audio playback");
        self.sink.pause();
        Ok(())
    }

    /// 再生を再開
    pub fn resume(&mut self) -> Result<()> {
        println!("Resuming audio playback");
        self.sink.play();
        Ok(())
    }

    /// 再生を停止
    pub fn stop(&mut self) -> Result<()> {
        println!("Stopping audio playback");
        self.stop_signal.store(true, Ordering::Relaxed);
        self.sink.stop();
        
        // デコーダースレッドの終了を待つ
        if let Some(thread) = self.decoder_thread.take() {
            let _ = thread.join();
        }
        
        Ok(())
    }

    /// 音量を設定 (0.0 - 1.0)
    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.original_volume = clamped_volume;

        if !self.is_muted.load(Ordering::Relaxed) {
            self.sink.set_volume(clamped_volume);
        }

        println!("Volume set to: {:.2}", clamped_volume);
        Ok(())
    }

    /// 現在の音量を取得
    pub fn volume(&self) -> f32 {
        if self.is_muted.load(Ordering::Relaxed) {
            0.0
        } else {
            self.original_volume
        }
    }

    /// ミュート
    pub fn mute(&mut self) -> Result<()> {
        println!("Muting audio");
        self.is_muted.store(true, Ordering::Relaxed);
        self.sink.set_volume(0.0);
        Ok(())
    }

    /// ミュート解除
    pub fn unmute(&mut self) -> Result<()> {
        println!("Unmuting audio");
        self.is_muted.store(false, Ordering::Relaxed);
        self.sink.set_volume(self.original_volume);
        Ok(())
    }

    /// ミュート切り替え
    pub fn toggle_mute(&mut self) -> Result<()> {
        if self.is_muted.load(Ordering::Relaxed) {
            self.unmute()
        } else {
            self.mute()
        }
    }

    /// 再生中かどうか
    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused()
    }

    /// ミュート中かどうか
    pub fn is_muted(&self) -> bool {
        self.is_muted.load(Ordering::Relaxed)
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
        if let Some(thread) = self.decoder_thread.take() {
            let _ = thread.join();
        }
    }
}

/// 音声デコードループ
fn decode_audio_loop(
    mut media_file: MediaFile,
    mut audio_decoder: AudioDecoder,
    sender: Sender<Vec<f32>>,
    stop_signal: Arc<AtomicBool>
) {
    println!("Audio decoder thread started");

    while !stop_signal.load(Ordering::Relaxed) {
        match media_file.read_packet() {
            Ok((stream, packet)) => {
                match audio_decoder.decode_next_frame(&packet) {
                    Ok(Some(audio_frame)) => {
                        // 音声フレームをf32サンプルに変換
                        match audio_frame.samples_as_f32() {
                            Ok(samples) => {
                                if sender.send(samples).is_err() {
                                    break; //受信側が終了
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to convert audio samples: {}", e);
                            }
                        }
                    }
                    Ok(None) => {
                        // フレームが得られなかった
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Audio decode error: {}", e);
                        continue;
                    }
                }
            }
            Err(_) => {
                // ストリーム終了
                break;
            }
        }
    }

    println!("Audio decoder thread finished");
} 

/// 音声システムの診断
pub fn diagnose_audio_system() -> Result<()> {
    println!("=== Audio System Diagnostics ===");
    
    // デフォルトの音声デバイスを試す
    match OutputStream::try_default() {
        Ok((_stream, _handle)) => {
            println!("✓ Default audio device is available");
        }
        Err(e) => {
            println!("✗ Default audio device failed: {}", e);
            
            // 可能な解決策を提案
            println!("\nPossible solutions:");
            println!("1. Check if audio service is running:");
            println!("   systemctl --user status pulseaudio");
            println!("2. Start PulseAudio:");
            println!("   pulseaudio --start");
            println!("3. Check audio devices:");
            println!("   aplay -l");
            println!("4. Test audio:");
            println!("   speaker-test -t wav -c 2");
            
            return Err(anyhow::anyhow!("Audio system not available"));
        }
    }
    
    println!("=== End Diagnostics ===");
    Ok(())
}
