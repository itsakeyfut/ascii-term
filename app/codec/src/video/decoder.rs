use std::sync::{Arc, Mutex};

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

/// 非同期ビデオデコーダー（tokio::task::spawn_blocking でエグゼキューターをブロックしない）
pub struct AsyncVideoDecoder {
    inner: Arc<Mutex<avio::VideoDecoder>>,
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

impl AsyncVideoDecoder {
    /// パスから非同期ビデオデコーダーを作成
    pub async fn open(path: &str) -> Result<Self> {
        let path = path.to_string();
        let decoder = tokio::task::spawn_blocking(move || {
            avio::VideoDecoder::open(&path)
                .output_format(PixelFormat::Rgb24)
                .build()
        })
        .await
        .map_err(|e| MediaError::Pipeline(format!("spawn_blocking panicked: {e}")))?
        .map_err(MediaError::Decode)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(decoder)),
            frame_count: 0,
        })
    }

    /// 次のフレームを非同期でデコード
    pub async fn decode_one(&mut self) -> Result<Option<VideoFrame>> {
        let inner = Arc::clone(&self.inner);

        let avio_frame = tokio::task::spawn_blocking(move || {
            inner
                .lock()
                .expect("VideoDecoder mutex not poisoned")
                .decode_one()
        })
        .await
        .map_err(|e| MediaError::Pipeline(format!("spawn_blocking panicked: {e}")))?
        .map_err(MediaError::Decode)?;

        match avio_frame {
            Some(frame) => {
                self.frame_count += 1;
                Ok(Some(VideoFrame::from_avio_frame(&frame)?))
            }
            None => Ok(None),
        }
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}
