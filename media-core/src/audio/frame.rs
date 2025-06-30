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
}