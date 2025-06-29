use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink};

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
        // オーディオストリームを初期化
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| anyhow::anyhow!("Failed to initialize audio stream: {}", e))?;

        // Sink を作成
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| anyhow::anyhow!("Failed to create audio sink: {}", e))?;

        // ファイルを開いてデコーダーを作成
        let file = File::open(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to open audio file: {}", e))?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| anyhow::anyhow!("Failed to decode audio file: {}", e))?;

        // 音源を Sink に追加
        sink.append(source);
        sink.pause(); // 最初は一時停止状態

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

        Ok(())
    }

    /// ミュート
    pub fn mute(&mut self) -> Result<()> {
        self.is_muted.store(true, Ordering::Relaxed);
        self.sink.set_volume(0.0);
        Ok(())
    }
}