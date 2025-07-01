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

    /// 次のフレームをデコード
    pub fn decode_next_frame(&mut self, packet: &ffmpeg::Packet) -> Result<Option<AudioFrame>> {
        if packet.stream() != self.stream_index {
            return Ok(None);
        }

        self.decoder.send_packet(packet)?;

        let mut frame = ffmpeg::frame::Audio::empty();
        let eagain = ffmpeg::ffi::AVERROR(ffmpeg::ffi::EAGAIN);
        match self.decoder.receive_frame(&mut frame) {
            Ok(()) => {
                let timestamp = self.pts_to_duration(frame.pts().unwrap_or(0));
                let pts = frame.pts().unwrap_or(0);

                let audio_frame = AudioFrame::from_ffmpeg_frame(&frame, timestamp, pts)?;

                self.frame_count += 1;
                Ok(Some(audio_frame))
            }
            Err(ffmpeg::Error::Other { errno }) if errno == eagain => {
                // もっとデータが必要
                Ok(None)
            }
            Err(e) => Err(MediaError::Ffmpeg(e)),
        }
    }

    /// 最後のフレームを取得（ストリーム終了時）
    pub fn flush(&mut self) -> Result<Vec<AudioFrame>> {
        self.decoder.send_eof()?;

        let mut frames = Vec::new();
        loop {
            let mut frame = ffmpeg::frame::Audio::empty();
            match self.decoder.receive_frame(&mut frame) {
                Ok(()) => {
                    let timestamp = self.pts_to_duration(frame.pts().unwrap_or(0));
                    let pts = frame.pts().unwrap_or(0);

                    let audio_frame = AudioFrame::from_ffmpeg_frame(&frame, timestamp, pts)?;
                    frames.push(audio_frame);
                    self.frame_count += 1;
                }
                Err(ffmpeg::Error::Eof) => break,
                Err(e) => return Err(MediaError::Ffmpeg(e)),
            }
        }

        Ok(frames)
    }

    /// PTSを時間に変換
    fn pts_to_duration(&self, pts: i64) -> Duration {
        let seconds = pts as f64 * self.time_base.numerator() as f64 / self.time_base.denominator() as f64;
        Duration::from_secs_f64(seconds)
    }

    /// デコーダー情報を取得
    pub fn sample_rate(&self) -> u32 {
        self.decoder.rate()
    }

    pub fn channels(&self) -> u16 {
        self.decoder.channels() as u16
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}