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
    media_file: Option<MediaFile>,
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
            media_file: None,
            frame_buffer: VecDeque::new(),
            is_running: Arc::new(AtomicBool::new(false)),
            is_eof: false,
        }
    }

    /// メディアファイルを設定
    pub fn set_media(&mut self, mut media_file: MediaFile) -> Result<()> {
        if media_file.info.has_video {
            self.decoder = Some(VideoDecoder::new(&media_file)?);
        }
        self.media_file = Some(media_file);
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
        Ok(())
    }

    /// 次のフレームを取得（実際のデコード処理を含む）
    pub fn next_frame(&mut self) -> Result<Option<VideoFrame>> {
        // バッファが空でない場合は、バッファから取得
        if let Some(frame) = self.frame_buffer.pop_front() {
            return Ok(Some(frame));
        }

        // EOFに達している場合は None を返す
        if self.is_eof {
            return Ok(None);
        }

        // バッファが空の場合は、新しいフレームをデコードしてバッファに追加
        self.decode_and_buffer_frames()?;

        // バッファから取得
        Ok(self.frame_buffer.pop_front())
    }

    /// フレームをデコードしてバッファに追加
    fn decode_and_buffer_frames(&mut self) -> Result<()> {
        let media_file = self
            .media_file
            .as_mut()
            .ok_or_else(|| MediaError::Pipeline("No media file set".to_string()))?;

        let decoder = self
            .decoder
            .as_mut()
            .ok_or_else(|| MediaError::Pipeline("No decoder available".to_string()))?;

        // 複数のパケットを処理してバッファを満たす
        while self.frame_buffer.len() < self.config.buffer_size && !self.is_eof {
            match media_file.read_packet() {
                Ok((stream, packet)) => {
                    // デコードを試行
                    match decoder.decode_next_frame(&packet) {
                        Ok(Some(frame)) => {
                            self.frame_buffer.push_back(frame);
                        }
                        Ok(None) => {
                            // フレームが得られなかった（まだデータが必要）
                            continue;
                        }
                        Err(e) => {
                            eprintln!("Frame decode error: {}", e);
                            continue;
                        }
                    }
                }
                Err(MediaError::Video(ref msg)) if msg == "End of stream" => {
                    // ストリーム終了
                    self.is_eof = true;

                    // デコーダーから残りのフレームを取得
                    match decoder.flush() {
                        Ok(remaining_frames) => {
                            for frame in remaining_frames {
                                self.frame_buffer.push_back(frame);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error flushing decoder: {}", e);
                        }
                    }
                    break;
                }
                Err(e) => {
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
