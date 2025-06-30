use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::errors::{MediaError, Result};
use crate::video::{VideoDecoder, VideoFrame};
use crate::media::MediaFile;

/// パイプライン設定
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub buffer_size: usize,
    pub enable_threading: bool,
    pub max_decode_threads: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10,
            enable_threading: true,
            max_decode_threads: 4,
        }
    }
}

/// メディア処理パイプライン
pub struct Pipeline {
    config: PipelineConfig,
    decoder: Option<VideoDecoder>,
    frame_buffer: VecDeque<VideoFrame>,
    is_running: Arc<AtomicBool>,
}

impl Pipeline {
    /// 新しいパイプラインを作成
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            decoder: None,
            frame_buffer: VecDeque::new(),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// メディアファイルを設定
    pub fn set_media(&mut self, media_file: &MediaFile) -> Result<()> {
        if media_file.info.has_video {
            self.decoder = Some(VideoDecoder::new(media_file)?);
        }
        Ok(())
    }

    /// パイプラインを開始
    pub fn start(&mut self) -> Result<()> {
        self.is_running.store(false, Ordering::Relaxed);
        self.frame_buffer.clear();
        Ok(())
    }

    /// パイプラインを停止
    pub fn stop(&mut self) -> Result<()> {
        self.is_running.store(false, Ordering::Relaxed);
        self.frame_buffer.clear();
        Ok(())
    }

    /// 次のフレームを取得
    pub fn next_frame(&mut self) -> Option<VideoFrame> {
        self.frame_buffer.pop_front()
    }

    /// バッファ内のフレーム数を取得
    pub fn buffer_size(&self) -> usize {
        self.frame_buffer.len()
    }

    /// バッファが満杯かどうか
    pub fn is_buffer_full(&self) -> bool {
        self.frame_buffer.len() >= self.config.buffer_size
    }

    /// パイプラインが実行中かどうか
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
}

/// パイプラインビルダー
pub struct PipelineBuilder {
    config: PipelineConfig,
}
