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

impl AudioDecoder {
    /// メディアファイルからオーディオデコーダーを作成
    pub fn new(media_file: &MediaFile) -> Result<Self> {
        let audio_stream = media_file
            .audio_stream()
            .ok_or_else(|| MediaError::Audio("No audio stream found".to_string()))?;

        let decoder = ffmpeg::codec::context::Context::from_parameters(audio_stream.parameters())?
            .decoder()
            .audio()?;

        Ok(Self {
            decoder,
            stream_index: audio_stream.index(),
            time_base: audio_stream.time_base(),
            frame_count: 0,
        })
    }
}