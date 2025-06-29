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
        self.is_muted.store(true, Ordering::Relaxed);
        self.sink.set_volume(0.0);
        Ok(())
    }

    /// ミュート解除
    pub fn unmute(&mut self) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    // テスト用のダミープレイヤーを作成
    fn create_dummy_player() -> AudioPlayer {
        // テスト環境では実際の音声ファイルが利用できない可能性があるため、
        // ここではダミーの実装を作成
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
    fn test_volume_control() {
        let mut player = create_dummy_player();

        assert!(player.set_volume(0.5).is_ok());
        assert_eq!(player.volume(), 0.5);
        
        assert!(player.mute().is_ok());
        assert_eq!(player.volume(), 0.0);
        assert!(player.is_muted());

        assert!(player.unmute().is_ok());
        assert_eq!(player.volume(), 0.5);
        assert!(!player.is_muted());
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