use std::collections::VecDeque;

use crate::audio::frame::{AudioFormat, AudioFrame};
use crate::errors::{MediaError, Result};

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
        // リサンプラーをリセット（設定が変更された場合）
        self.resampler = None;
    }

    /// フレームを処理してバッファに追加
    pub fn process_frame(&mut self, frame: AudioFrame) -> Result<()> {
        let mut processed_frame = frame;

        // リサンプリングが必要かチェック
        if processed_frame.sample_rate != self.config.output_sample_rate
            || processed_frame.channels != self.config.output_channels
            || processed_frame.format != self.config.output_format
        {
            processed_frame = self.resample_frame(processed_frame)?;
        }

        // 音量調整
        if self.config.volume != 1.0 || self.config.muted {
            processed_frame = self.apply_volume(processed_frame)?;
        }

        // バッファに追加
        self.buffer.push_back(processed_frame);

        // バッファサイズを制限
        while self.buffer.len() > self.config.buffer_size {
            self.buffer.pop_front();
        }

        Ok(())
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

    /// ミュート状態を設定
    pub fn set_muted(&mut self, muted: bool) {
        self.config.muted = muted;
    }

    /// フレームをリサンプリング
    fn resample_frame(&mut self, frame: AudioFrame) -> Result<AudioFrame> {
        // リサンプラーを初期化（必要に応じて）
        if self.resampler.is_none() {
            self.resampler = Some(SimpleResampler::new(
                frame.sample_rate,
                frame.channels,
                frame.format,
                self.config.output_sample_rate,
                self.config.output_channels,
                self.config.output_format,
            )?);
        }

        if let Some(resampler) = &mut self.resampler {
            resampler.resample(frame)
        } else {
            Ok(frame)
        }
    }

    /// 音量を適用
    fn apply_volume(&self, mut frame: AudioFrame) -> Result<AudioFrame> {
        if self.config.muted {
            // ミュート：すべてのサンプルを0にする
            frame.data.fill(0);
            return Ok(frame);
        }

        if self.config.volume == 1.0 {
            return Ok(frame);
        }

        // 音量調整は浮動小数点で行うのが最も正確
        let samples = frame.samples_as_f32()?;
        let adjusted_samples: Vec<f32> = samples
            .iter()
            .map(|&sample| sample * self.config.volume)
            .collect();

        // バイトデータに戻す
        let mut new_data = Vec::with_capacity(adjusted_samples.len() * 4);
        for sample in adjusted_samples {
            new_data.extend_from_slice(&sample.to_le_bytes());
        }

        frame.data = new_data;
        frame.format = AudioFormat::F32LE;

        Ok(frame)
    }
}

/// 簡易理リサンプラー
struct SimpleResampler {
    input_sample_rate: u32,
    input_channels: u16,
    input_format: AudioFormat,
    output_sample_rate: u32,
    output_channels: u16,
    output_format: AudioFormat,
    ratio: f64,
}

impl SimpleResampler {
    /// 新しいサンプラーを作成
    fn new(
        input_sample_rate: u32,
        input_channels: u16,
        input_format: AudioFormat,
        output_sample_rate: u32,
        output_channels: u16,
        output_format: AudioFormat,
    ) -> Result<Self> {
        let ratio = output_sample_rate as f64 / input_sample_rate as f64;

        Ok(Self {
            input_sample_rate,
            input_channels,
            input_format,
            output_sample_rate,
            output_channels,
            output_format,
            ratio,
        })
    }

    /// フレームをリサンプル
    fn resample(&self, frame: AudioFrame) -> Result<AudioFrame> {
        // 入力フレームを浮動小数点配列に変換
        let input_samples = frame.samples_as_f32()?;

        // チャンネル数の変換
        let mono_samples = if self.input_channels != self.output_channels {
            self.convert_channels(&input_samples, frame.samples)?
        } else {
            input_samples
        };

        // サンプルレート変換
        let resampled_samples = if self.input_sample_rate != self.output_sample_rate {
            self.convert_sample_rate(&mono_samples)?
        } else {
            mono_samples
        };

        // 出力形式に変換
        let output_data = self.convert_format(&resampled_samples)?;
        let output_samples = resampled_samples.len() / self.output_channels as usize;

        Ok(AudioFrame::new(
            output_data,
            output_samples,
            self.output_channels,
            self.output_sample_rate,
            self.output_format,
            frame.timestamp,
            frame.pts,
            false,
        ))
    }

    fn convert_channels(&self, samples: &[f32], input_samples: usize) -> Result<Vec<f32>> {
        if self.input_channels == 1 && self.output_channels == 2 {
            // モノラル → ステレオ
            let mut output = Vec::with_capacity(samples.len() * 2);
            for &sample in samples {
                output.push(sample); // 左チャンネル
                output.push(sample); // 右チャンネル
            }
            Ok(output)
        } else if self.input_channels == 2 && self.output_channels == 1 {
            // ステレオ → モノラル
            let mut output = Vec::with_capacity(input_samples);
            for chunk in samples.chunks_exact(2) {
                let mono_sample = (chunk[0] + chunk[1]) / 2.0;
                output.push(mono_sample);
            }
            Ok(output)
        } else if self.input_channels == self.output_channels {
            Ok(samples.to_vec())
        } else {
            // その他の変換は簡略化
            Ok(samples.to_vec())
        }
    }

    fn convert_sample_rate(&self, samples: &[f32]) -> Result<Vec<f32>> {
        if self.ratio == 1.0 {
            return Ok(samples.to_vec());
        }

        let input_frames = samples.len() / self.output_channels as usize;
        let output_frames = (input_frames as f64 * self.ratio).round() as usize;
        let mut output = Vec::with_capacity(output_frames * self.output_channels as usize);

        // 線形補間による簡易リサンプリング
        for output_frame in 0..output_frames {
            let input_frame_f = output_frame as f64 / self.ratio;
            let input_frame = input_frame_f.floor() as usize;
            let fraction = input_frame_f - input_frame as f64;

            for channel in 0..self.output_channels {
                let idx1 = input_frame * self.output_channels as usize + channel as usize;
                let idx2 = ((input_frame + 1).min(input_frames - 1))
                    * self.output_channels as usize
                    + channel as usize;

                if idx1 < samples.len() && idx2 < samples.len() {
                    let sample1 = samples[idx1];
                    let sample2 = samples[idx2];
                    let interpolated = sample1 + (sample2 - sample1) * fraction as f32;
                    output.push(interpolated);
                } else if idx1 < samples.len() {
                    output.push(samples[idx1]);
                } else {
                    output.push(0.0);
                }
            }
        }

        Ok(output)
    }

    fn convert_format(&self, samples: &[f32]) -> Result<Vec<u8>> {
        let mut output = Vec::new();

        match self.output_format {
            AudioFormat::F32LE => {
                for &sample in samples {
                    output.extend_from_slice(&sample.to_le_bytes());
                }
            }
            AudioFormat::S16LE => {
                for &sample in samples {
                    let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                    output.extend_from_slice(&sample_i16.to_le_bytes());
                }
            }
            AudioFormat::S32LE => {
                for &sample in samples {
                    let sample_i32 = (sample.clamp(-1.0, 1.0) * 2147483647.0) as i32;
                    output.extend_from_slice(&sample_i32.to_le_bytes());
                }
            }
            _ => {
                return Err(MediaError::Audio("Unsupported output format".to_string()));
            }
        }

        Ok(output)
    }
}
