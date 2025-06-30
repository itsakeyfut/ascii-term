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
