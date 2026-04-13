use crate::audio::frame::AudioFrame;
use crate::errors::{MediaError, Result};

/// オーディオデコーダー
pub struct AudioDecoder {
    inner: avio::AudioDecoder,
    frame_count: u64,
}

impl AudioDecoder {
    /// パスからオーディオデコーダーを作成
    pub fn new(path: &str) -> Result<Self> {
        let inner = avio::AudioDecoder::open(path)
            .build()
            .map_err(MediaError::Decode)?;

        Ok(Self {
            inner,
            frame_count: 0,
        })
    }

    /// 次のフレームをデコード
    pub fn decode_one(&mut self) -> Result<Option<AudioFrame>> {
        match self.inner.decode_one() {
            Ok(Some(frame)) => {
                let audio_frame = AudioFrame::from_avio_frame(&frame)?;
                self.frame_count += 1;
                Ok(Some(audio_frame))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(MediaError::Decode(e)),
        }
    }

    /// デコード済みフレーム数を取得
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}
