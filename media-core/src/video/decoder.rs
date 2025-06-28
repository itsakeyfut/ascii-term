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

impl VideoDecoder {
    /// メディアファイルからビデオデコーダーを作成
    pub fn new(media_file: &MediaFile) -> Result<Self> {
        let video_stream = media_file
            .video_stream()
            .ok_or_else(|| MediaError::Video("No video stream found".to_string()))?;

        let decoder = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())?
            .decoder()
            .video()?;

        Ok(Self {
            decoder,
            stream_index: video_stream.index(),
            time_base: video_stream.time_base(),
            frame_count: 0,
        })
    }

    /// PTS を時間に変換
    fn pts_to_duration(&self, pts: i64) -> Duration {
        let seconds = pts as f64 * self.time_base.numerator() as f64 / self.time_base.denominator() as f64;
        Duration::from_secs_f64(seconds)
    }

    /// フレームを RGB24 に変換
    fn convert_to_rgb24(&self, frame: &ffmpeg::frame::Video) -> Result<ffmpeg::frame::Video> {
        let mut converter = ffmpeg::software::scaling::context::Context::get(
            frame.format(),
            frame.width(),
            frame.height(),
            ffmpeg::format::Pixel::RGB24,
            frame.width(),
            frame.height(),
            ffmpeg::software::scaling::flag::Flags::BILINEAR,
        )?;

        let mut rgb_frame = ffmpeg::frame::Video::empty();
        converter.run(frame, &mut rgb_frame)?;

        Ok(rgb_frame)
    }
}