use std::time::Duration;

use ffmpeg_next as ffmpeg;

use crate::errors::{MediaError, Result};
use crate::media::MediaFile;
use crate::video::frame::{VideoFrame, FrameFormat};

/// ビデオデコーダー
pub struct VideoDecoder {
    decoder: ffmpeg::decoder::Video,
    stream_index: usize,
    time_base: ffmpeg::Rational,
    frame_count: u64,
}
