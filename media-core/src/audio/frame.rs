use std::time::Duration;

use ffmpeg_next::format::sample;

use crate::errors::{MediaError, Result};

/// オーディオフレームのサンプル形式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioFormat {
    /// 8ビット符号なし整数
    U8,
    /// 16ビット符号付き整数（リトルエンディアン）
    S16LE,
    /// 16ビット符号付き整数（ビッグエンディアン）
    S16BE,
    /// 24ビット符号付き整数
    S24LE,
    /// 32ビット符号付き整数
    S32LE,
    /// 32ビット浮動小数点
    F32LE,
    /// 64ビット浮動小数点
    F64LE,
}

impl AudioFormat {
    /// サンプルあたりのバイト数を取得
    pub fn bytes_per_sample(&self) -> usize {
        match self {
            AudioFormat::U8 => 1,
            AudioFormat::S16LE | AudioFormat::S16BE => 2,
            AudioFormat::S24LE => 3,
            AudioFormat::S32LE | AudioFormat::F32LE => 4,
            AudioFormat::F64LE => 8,
        }
    }

    /// 浮動小数点形式かどうか
    pub fn is_float(&self) -> bool {
        matches!(self, AudioFormat::F32LE | AudioFormat::F64LE)
    }

    /// FFmpeg のサンプル形式から変換
    pub fn from_ffmpeg_format(format: ffmpeg_next::format::Sample) -> Result<Self> {
        match format {
            ffmpeg_next::format::Sample::U8(_) => Ok(AudioFormat::U8),
            ffmpeg_next::format::Sample::I16(_) => Ok(AudioFormat::S16LE),
            ffmpeg_next::format::Sample::I32(_) => Ok(AudioFormat::S32LE),
            ffmpeg_next::format::Sample::F32(_) => Ok(AudioFormat::F32LE),
            ffmpeg_next::format::Sample::F64(_) => Ok(AudioFormat::F64LE),
            _ => Err(MediaError::UnsupportedCodec(format!("Unsupported audio format: {:?}", format))),
        }
    }
}

/// オーディオフレームを表現する構造体
#[derive(Debug, Clone)]
pub struct AudioFrame {
    /// オーディオデータ（インターリーブまたはプレーナー）
    pub data: Vec<u8>,
    /// サンプル数
    pub samples: usize,
    /// チャンネル数
    pub channels: u16,
    /// サンプルレート（Hz）
    pub sample_rate: u32,
    /// サンプル形式
    pub format: AudioFormat,
    /// タイムスタンプ
    pub timestamp: Duration,
    /// PTS (Presentation Time Stamp)
    pub pts: i64,
    /// データがプレーナー形式かどうか
    pub is_planar: bool,
}

impl AudioFrame {
    /// 新しいフレームを作成
    pub fn new(
        data: Vec<u8>,
        samples: usize,
        channels: u16,
        sample_rate: u32,
        format: AudioFormat,
        timestamp: Duration,
        pts: i64,
        is_planar: bool,
    ) -> Self {
        Self {
            data,
            samples,
            channels,
            sample_rate,
            format,
            timestamp,
            pts,
            is_planar,
        }
    }

    /// FFmpeg のオーディオフレームから作成
    pub fn from_ffmpeg_frame(
        frame: &ffmpeg_next::frame::Audio,
        timestamp: Duration,
        pts: i64,
    ) -> Result<Self> {
        let samples = frame.samples();
        let channels = frame.channels() as u16;
        let sample_rate = frame.rate();
        let format = AudioFormat::from_ffmpeg_format(frame.format())?;
        let is_planar = frame.is_planar();

        // フレームデータをコピー
        let mut data = Vec::new();
        if is_planar {
            // プレーナー形式：各チャンネルが別々のプレーンに格納
            for plane in 0..frame.planes() {
                let plane_data = frame.data(plane);
                data.extend_from_slice(plane_data);
            }
        } else {
            // インターリーブ形式：全チャンネルが混在
            let plane_data = frame.data(0);
            data.extend_from_slice(plane_data);
        }

        Ok(Self::new(
            data,
            samples,
            channels,
            sample_rate,
            format,
            timestamp,
            pts,
            is_planar,
        ))
    }

    /// フレームの長さ（時間）を取得
    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.samples as f64 / self.sample_rate as f64)
    }

    /// 総バイト数を取得
    pub fn total_bytes(&self) -> usize {
        self.samples * self.channels as usize * self.format.bytes_per_sample()
    }

    /// インターリーブ形式に変換
    pub fn to_interleaved(&self) -> Result<AudioFrame> {
        if !self.is_planar {
            return Ok(self.clone());
        }

        let bytes_per_sample = self.format.bytes_per_sample();
        let mut interleaved_data = Vec::with_capacity(self.data.len());

        // プレーナーからインターリーブに変換
        for sample_idx in 0..self.samples {
            for channel in 0..self.channels {
                let plane_offset = channel as usize * self.samples * bytes_per_sample;
                let sample_offset = sample_idx * bytes_per_sample;
                let start = plane_offset + sample_offset;
                let end = start + bytes_per_sample;

                if end <= self.data.len() {
                    interleaved_data.extend_from_slice(&self.data[start..end]);
                }
            }
        }

        Ok(AudioFrame::new(
            interleaved_data,
            self.samples,
            self.channels,
            self.sample_rate,
            self.format,
            self.timestamp,
            self.pts,
            false, // インターリーブ形式
        ))
    }

    /// プレーナー形式に変換
    pub fn to_planar(&self) -> Result<AudioFrame> {
        if !self.is_planar {
            return Ok(self.clone());
        }

        let bytes_per_sample = self.format.bytes_per_sample();
        let mut planar_data = Vec::with_capacity(self.data.len());

        // インターリーブからプレーナーに変換
        for channel in 0..self.channels {
            for sample_idx in 0..self.samples {
                let interleaved_offset = (sample_idx * self.channels as usize + channel as usize) * bytes_per_sample;
                let start = interleaved_offset;
                let end = start + bytes_per_sample;

                if end <= self.data.len() {
                    planar_data.extend_from_slice(&self.data[start..end]);
                }
            }
        }

        Ok(AudioFrame::new(
            planar_data,
            self.samples,
            self.channels,
            self.sample_rate,
            self.format,
            self.timestamp,
            self.pts,
            true, // プレーナー形式
        ))
    }

    /// サンプルを浮動小数点配列として取得（正規化済み）
    pub fn samples_as_f32(&self) -> Result<Vec<f32>> {
        let mut samples = Vec::with_capacity(self.samples * self.channels as usize);

        match self.format {
            AudioFormat::U8 => {
                for &byte in &self.data {
                    samples.push((byte as f32 - 128.0) / 128.0);
                }
            }
            AudioFormat::S16LE => {
                for chunk in self.data.chunks_exact(2) {
                    let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                    samples.push(sample as f32 / 32768.0);
                }
            }
            AudioFormat::S32LE => {
                for chunk in self.data.chunks_exact(4) {
                    let sample = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    samples.push(sample as f32 / 2147483648.0);
                }
            }
            AudioFormat::F32LE => {
                for chunk in self.data.chunks_exact(4) {
                    let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    samples.push(sample);
                }
            }
            _ => {
                return Err(MediaError::Audio("Unsupported format for f32 conversion".to_string()));
            }
        }

        Ok(samples)
    }
}