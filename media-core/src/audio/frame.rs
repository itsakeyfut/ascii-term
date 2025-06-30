use std::time::Duration;

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
    pub format: Duration,
    /// タイムスタンプ
    pub timestamp: Duration,
    /// PTS (Presentation Time Stamp)
    pub pts: i64,
    /// データがプレーナー形式かどうか
    pub is_planar: bool,
}