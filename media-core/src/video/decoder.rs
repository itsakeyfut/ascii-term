use std::time::Duration;

use ffmpeg_next as ffmpeg;

use crate::errors::{MediaError, Result};
use crate::media::MediaFile;
use crate::video::frame::{FrameFormat, VideoFrame};

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

    /// 次のフレームをデコード
    pub fn decode_next_frame(&mut self, packet: &ffmpeg::Packet) -> Result<Option<VideoFrame>> {
        if packet.stream() != self.stream_index {
            return Ok(None);
        }

        self.decoder.send_packet(packet)?;

        let mut frame = ffmpeg::frame::Video::empty();
        let eagain = ffmpeg::ffi::AVERROR(ffmpeg::ffi::EAGAIN);
        match self.decoder.receive_frame(&mut frame) {
            Ok(()) => {
                let timestamp = self.pts_to_duration(frame.pts().unwrap_or(0));
                let pts = frame.pts().unwrap_or(0);

                // フレームを RGB24 に変換
                let rgb_frame = self.convert_to_rgb24(&frame)?;
                let video_frame = VideoFrame::from_ffmpeg_frame(&rgb_frame, timestamp, pts)?;

                self.frame_count += 1;
                Ok(Some(video_frame))
            }
            Err(ffmpeg::Error::Other { errno }) if errno == eagain => {
                // もっとデータが必要
                Ok(None)
            }
            Err(e) => Err(MediaError::Ffmpeg(e)),
        }
    }

    /// 最後のフレームを取得（ストリーム終了時）
    pub fn flush(&mut self) -> Result<Vec<VideoFrame>> {
        self.decoder.send_eof()?;

        let mut frames = Vec::new();
        loop {
            let mut frame = ffmpeg::frame::Video::empty();
            match self.decoder.receive_frame(&mut frame) {
                Ok(()) => {
                    let timestamp = self.pts_to_duration(frame.pts().unwrap_or(0));
                    let pts = frame.pts().unwrap_or(0);

                    let rgb_frame = self.convert_to_rgb24(&frame)?;
                    let video_frame = VideoFrame::from_ffmpeg_frame(&rgb_frame, timestamp, pts)?;
                    frames.push(video_frame);
                    self.frame_count += 1;
                }
                Err(ffmpeg::Error::Eof) => break,
                Err(e) => return Err(MediaError::Ffmpeg(e)),
            }
        }

        Ok(frames)
    }

    /// PTS を時間に変換
    fn pts_to_duration(&self, pts: i64) -> Duration {
        let seconds =
            pts as f64 * self.time_base.numerator() as f64 / self.time_base.denominator() as f64;
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

    /// デコーダーの情報を取得
    pub fn width(&self) -> u32 {
        self.decoder.width()
    }

    pub fn height(&self) -> u32 {
        self.decoder.height()
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn codec_name(&self) -> String {
        format!("{:?}", self.decoder.id())
    }
}
