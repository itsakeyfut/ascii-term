use avio::PixelFormat;

use crate::errors::{MediaError, Result};
use crate::video::frame::VideoFrame;

/// ビデオデコーダー
pub struct VideoDecoder {
    inner: avio::VideoDecoder,
    width: u32,
    height: u32,
    frame_count: u64,
}

impl VideoDecoder {
    /// パスからビデオデコーダーを作成
    pub fn new(path: &str, width: u32, height: u32) -> Result<Self> {
        let inner = avio::VideoDecoder::open(path)
            .output_format(PixelFormat::Rgb24)
            .build()
            .map_err(MediaError::Decode)?;

        Ok(Self {
            inner,
            width,
            height,
            frame_count: 0,
        })
    }

    /// 次のフレームをデコード
    pub fn decode_one(&mut self) -> Result<Option<VideoFrame>> {
        match self.inner.decode_one() {
            Ok(Some(frame)) => {
                let video_frame = VideoFrame::from_avio_frame(&frame)?;
                self.frame_count += 1;
                Ok(Some(video_frame))
            }
            Ok(None) => Ok(None),
            Err(avio::DecodeError::EndOfStream) => Ok(None),
            Err(e) => Err(MediaError::Decode(e)),
        }
    }

    /// デコーダーの情報を取得
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}
