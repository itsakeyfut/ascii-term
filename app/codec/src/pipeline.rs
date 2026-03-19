use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::errors::{MediaError, Result};
use crate::media::MediaFile;
use crate::video::{VideoDecoder, VideoFrame};

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
    is_eof: bool,
}

impl Pipeline {
    /// 新しいパイプラインを作成
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            decoder: None,
            frame_buffer: VecDeque::new(),
            is_running: Arc::new(AtomicBool::new(false)),
            is_eof: false,
        }
    }

    /// メディアファイルを設定
    pub fn set_media(&mut self, media_file: MediaFile) -> Result<()> {
        if media_file.info.has_video {
            let width = media_file.info.width.unwrap_or(0);
            let height = media_file.info.height.unwrap_or(0);
            self.decoder = Some(VideoDecoder::new(&media_file.path, width, height)?);
        }
        self.is_eof = false;
        Ok(())
    }

    /// パイプラインを開始
    pub fn start(&mut self) -> Result<()> {
        self.is_running.store(true, Ordering::Relaxed);
        self.frame_buffer.clear();
        self.is_eof = false;
        Ok(())
    }

    /// パイプラインを停止
    pub fn stop(&mut self) -> Result<()> {
        self.is_running.store(false, Ordering::Relaxed);
        self.frame_buffer.clear();
        self.decoder = None;
        Ok(())
    }

    /// 次のフレームを取得（実際のデコード処理を含む）
    pub fn next_frame(&mut self) -> Result<Option<VideoFrame>> {
        if let Some(frame) = self.frame_buffer.pop_front() {
            return Ok(Some(frame));
        }

        if self.is_eof {
            return Ok(None);
        }

        self.decode_and_buffer_frames()?;

        Ok(self.frame_buffer.pop_front())
    }

    /// フレームをデコードしてバッファに追加
    fn decode_and_buffer_frames(&mut self) -> Result<()> {
        let decoder = self
            .decoder
            .as_mut()
            .ok_or_else(|| MediaError::Pipeline("No decoder available".to_string()))?;

        while self.frame_buffer.len() < self.config.buffer_size && !self.is_eof {
            match decoder.decode_one() {
                Ok(Some(frame)) => {
                    self.frame_buffer.push_back(frame);
                }
                Ok(None) => {
                    self.is_eof = true;
                    break;
                }
                Err(e) => {
                    eprintln!("Frame decode error: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
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

    /// ストリームが終了しているかどうか
    pub fn is_eof(&self) -> bool {
        self.is_eof
    }

    /// 完全に終了しているかどうか（EOFかつバッファが空）
    pub fn is_finished(&self) -> bool {
        self.is_eof && self.frame_buffer.is_empty()
    }
}

/// パイプラインビルダー
pub struct PipelineBuilder {
    config: PipelineConfig,
}

impl PipelineBuilder {
    /// 新しいビルダーを作成
    pub fn new() -> Self {
        Self {
            config: PipelineConfig::default(),
        }
    }

    /// バッファサイズを設定
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// スレッド使用を設定
    pub fn enable_threading(mut self, enable: bool) -> Self {
        self.config.enable_threading = enable;
        self
    }

    /// 最大デコードスレッド数を設定
    pub fn max_decode_threads(mut self, threads: usize) -> Self {
        self.config.max_decode_threads = threads;
        self
    }

    /// パイプラインを構築
    pub fn build(self) -> Pipeline {
        Pipeline::new(self.config)
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = PipelineBuilder::new().buffer_size(5).build();

        assert_eq!(pipeline.config.buffer_size, 5);
        assert_eq!(pipeline.buffer_size(), 0);
        assert!(!pipeline.is_running());
        assert!(!pipeline.is_eof());
    }

    #[test]
    fn test_pipeline_state() {
        let mut pipeline = Pipeline::new(PipelineConfig::default());

        assert!(!pipeline.is_running());
        assert!(!pipeline.is_finished());

        pipeline.start().unwrap();
        assert!(pipeline.is_running());

        pipeline.stop().unwrap();
        assert!(!pipeline.is_running());
    }
}
