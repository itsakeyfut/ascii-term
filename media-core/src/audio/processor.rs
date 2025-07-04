use std::collections::VecDeque;

use crate::errors::{MediaError, Result};
use crate::audio::frame::{AudioFrame, AudioFormat};

/// オーディオ処理設定
pub struct AudioProcessorConfig {
    /// 出力サンプルレート
    pub output_sample_rate: u32,
    /// 出力チャンネル数
    pub output_channels: u16,
    /// 出力サンプル形式
    pub output_format: AudioFormat,
    /// バッファサイズ
    pub buffer_size: usize,
    /// 音量（0.0-1.0）
    pub volume: f32,
    /// ミュート状態
    pub muted: bool,
}

impl Default for AudioProcessorConfig {
    fn default() -> Self {
        Self {
            output_sample_rate: 44100,
            output_channels: 2,
            output_format: AudioFormat::F32LE,
            buffer_size: 4096,
            volume: 1.0,
            muted: false,
        }
    }
}

/// オーディオプロセッサー
pub struct AudioProcessor {
    config: AudioProcessorConfig,
    buffer: VecDeque<AudioFrame>,
    resampler: Option<SimpleResampler>,
}

impl AudioProcessor {
    /// 新しいオーディオプロセッサーを作成
    pub fn new(config: AudioProcessorConfig) -> Self {
        Self {
            config,
            buffer: VecDeque::new(),
            resampler: None,
        }
    }

    /// 設定を更新
    pub fn update_config(&mut self, config: AudioProcessorConfig) {
        self.config = config;
        // 理サンプラーをリセット（設定が変更された場合）
        self.resampler = None;
    }

    /// 次のフレームを取得
    pub fn next_frame(&mut self) -> Option<AudioFrame> {
        self.buffer.pop_front()
    }

    /// バッファ内のフレーム数
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// バッファをクリア
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// 音量を設定
    pub fn set_volume(&mut self, volume: f32) {
        self.config.volume = volume.clamp(0.0, 2.0);
    }
}

/// 簡易理サンプラー
struct SimpleResampler {
    input_sample_rate: u32,
    input_channels: u16,
    input_format: AudioFormat,
    output_sample_rate: u32,
    output_channels: u16,
    output_format: AudioFormat,
    ratio: f64,
}