use std::fs::File;
use std::io::BufReader;
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

/// オーディオプレイヤー
pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
    is_muted: Arc<AtomicBool>,
    original_volume: f32,
}

impl AudioPlayer {
    /// 新しいオーディオプレイヤーを作成
    pub fn new(file_path: &str) -> Result<Self> {
        println!("Initializing audio player for: {}", file_path);

        // オーディオストリームを初期化
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| {
                eprintln!("Failed to initialize audio stream: {}", e);
                eprintln!("This might be due to:");
                eprintln!("1. No audio device available");
                eprintln!("2. Audio system not running (try: pulseaudio --start)");
                eprintln!("3. WSL environment needs additional setup");
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

        // ファイルを開いてデコーダーを作成
        let file = File::open(file_path)
            .map_err(|e| {
                eprintln!("Failed to open audio file '{}': {}", file_path, e);
                anyhow::anyhow!("Failed to open audio file: {}", e)
            })?;

        println!("Audio file opened successfully");

        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| {
                eprintln!("Failed to decode audio file '{}': {}", file_path, e);
                eprintln!("Supported formats: MP3, WAV, FLAC, OGG, etc.");
                anyhow::anyhow!("Failed to decode audio file: {}", e)
            })?;

        println!("Audio decoder created successfully");

        // 音源を Sink に追加
        sink.append(source);
        sink.pause(); // 最初は一時停止状態

        println!("Audio player initialized successfully");

        Ok(Self {
            _stream,
            sink,
            is_muted: Arc::new(AtomicBool::new(false)),
            original_volume: 1.0,
        })
    }

    /// 再生を開始
    pub fn play(&mut self) -> Result<()> {
        self.sink.play();
        Ok(())
    }

    /// 再生を一時停止
    pub fn pause(&mut self) -> Result<()> {
        self.sink.pause();
        Ok(())
    }

    /// 再生を再開
    pub fn resume(&mut self) -> Result<()> {
        self.sink.play();
        Ok(())
    }

    /// 再生を停止
    pub fn stop(&mut self) -> Result<()> {
        self.sink.stop();
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

    /// 再生位置を先頭に戻す
    /// 
    /// 簡易実装のため、見直しが必要
    pub fn seek_to_start(&mut self, file_path: &str) -> Result<()> {
        println!("Seeking to start of audio file");

        // 現在の音源をクリア
        self.sink.stop();

        // 新しい音源を読み込み
        let file = File::open(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to open audio file: {}", e))?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| anyhow::anyhow!("Failed to decode audio file: {}", e))?;

        // 音源を Sink に追加
        self.sink.append(source);

        // 音量とミュート状態を復元
        if self.is_muted.load(Ordering::Relaxed) {
            self.sink.set_volume(0.0);
        } else {
            self.sink.set_volume(self.original_volume);
        }

        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // テスト用のダミープレイヤーを作成
    fn create_dummy_player() -> AudioPlayer {
        // テスト環境では実際の音声ファイルが利用できない可能性があるため、
        // ここではダミーの実装を作成
        // 特に仮想環境では音声機能が備わっていない可能性があるため、テストが失敗する
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        AudioPlayer {
            _stream,
            sink,
            is_muted: Arc::new(AtomicBool::new(false)),
            original_volume: 1.0,
        }
    }

    #[test]
    fn test_audio_system_availability() {
        // 音声システムの可用性をテスト
        match OutputStream::try_default() {
            Ok(_) => println!("Audio system is available for testing"),
            Err(e) => println!("Audio system not available: {}", e),
        }
    }



    #[test]
    fn test_volume_control() {
        // 音声システムが利用可能な場合のみテスト
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                let mut player = AudioPlayer {
                    _stream,
                    sink,
                    is_muted: Arc::new(AtomicBool::new(false)),
                    original_volume: 1.0,
                };

                assert!(player.set_volume(0.5).is_ok());
                assert_eq!(player.volume(), 0.5);
                
                assert!(player.mute().is_ok());
                assert_eq!(player.volume(), 0.0);
                assert!(player.is_muted());

                assert!(player.unmute().is_ok());
                assert_eq!(player.volume(), 0.5);
                assert!(!player.is_muted());
            }
        }
    }

    #[test]
    fn test_mute_toggle() {
        let mut player = create_dummy_player();

        assert!(!player.is_muted());

        assert!(player.toggle_mute().is_ok());
        assert!(player.is_muted());

        assert!(player.toggle_mute().is_ok());
        assert!(!player.is_muted());
    }
}