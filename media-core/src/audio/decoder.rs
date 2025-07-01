use std::time::Duration;

use ffmpeg_next as ffmpeg;

use crate::errors::{MediaError, Result};
use crate::media::MediaFile;
use crate::audio::frame::{AudioFrame, AudioFormat};

/// オーディオデコーダー
pub struct AudioDecoder {
    decoder: ffmpeg::decoder::Audio,
    stream_index: usize,
    time_base: ffmpeg::Rational,
    frame_count: u64,
}
