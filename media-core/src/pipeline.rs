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
