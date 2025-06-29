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
