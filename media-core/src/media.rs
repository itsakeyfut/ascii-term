use std::path::Path;
use std::time::Duration;

use ffmpeg_next as ffmpeg;

use crate::errors::{MediaError, Result};

/// メディアファイルの種類を表す列挙型
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Video,
    Audio,
    Image,
    Unknown,
}

/// メディアファイルの情報を保持する構造体
#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub duration: Option<Duration>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub has_video: bool,
    pub has_audio: bool,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
}

impl Default for MediaInfo {
    fn default() -> Self {
        Self {
            duration: None,
            width: None,
            height: None,
            fps: None,
            has_video: false,
            has_audio: false,
            video_codec: None,
            audio_codec: None,
            sample_rate: None,
            channels: None,
        }
    }
}

/// メディアファイルを表現する構造体
pub struct MediaFile {
    pub path: String,
    pub media_type: MediaType,
    pub info: MediaInfo,
    format_context: ffmpeg::format::context::Input,
}

impl MediaFile {
    /// ファイルパスからメディアファイルを開く
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_str()
            .ok_or_else(|| MediaError::InvalidFormat("Invalid path".to_string()))?;

        let format_context = ffmpeg::format::input(&path_str)
            .map_err(|e| MediaError::Ffmpeg(e))?;

        let info = Self::extract_media_info(&format_context)?;
        let media_type = Self::determine_media_type(&info);

        Ok(MediaFile {
            path: path_str.to_string(),
            media_type,
            info,
            format_context,
        })
    }

    /// メディア情報を抽出
    fn extract_media_info(format_context: &ffmpeg::format::context::Input) -> Result<MediaInfo> {
        let mut info = MediaInfo::default();

        // 全体の長さ
        if format_context.duration() != ffmpeg::ffi::AV_NOPTS_VALUE {
            info.duration = Some(Duration::from_micros(
                (format_context.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64 * 1_000_000.0) as u64
            ));
        }

        // ストリーム情報を解析
        for stream in format_context.streams() {
            match stream.parameters().medium() {
                ffmpeg::media::Type::Video => {
                    info.has_video = true;
                    let params = stream.parameters();
                    info.width = Some(params.width());
                    info.height = Some(params.height());
                    
                    // フレームレート計算
                    let time_base = stream.time_base();
                    let avg_frame_rate = stream.avg_frame_rate();
                    if avg_frame_rate.numerator() > 0 && avg_frame_rate.denominator() > 0 {
                        info.fps = Some(avg_frame_rate.numerator() as f64 / avg_frame_rate.denominator() as f64);
                    }

                    // ビデオコーデック
                    info.video_codec = Some(format!("{:?}", params.id()));
                }
                ffmpeg::media::Type::Audio => {
                    info.has_audio = true;
                    let params = stream.parameters();
                    info.sample_rate = Some(params.rate());
                    info.channels = Some(params.channels());
                    
                    // オーディオコーデック
                    info.audio_codec = Some(format!("{:?}", params.id()));
                }
                _ => {}
            }
        }

        Ok(info)
    }

    /// メディアタイプを判定
    fn determine_media_type(info: &MediaInfo) -> MediaType {
        match (info.has_video, info.has_audio) {
            (true, _) => MediaType::Video,
            (false, true) => MediaType::Audio,
            (false, false) => MediaType::Unknown,
        }
    }

    /// ビデオストリームを取得
    pub fn video_stream(&self) -> Option<ffmpeg::Stream> {
        self.format_context
            .streams()
            .best(ffmpeg::media::Type::Video)
    }

    /// オーディオストリームを取得
    pub fn audio_stream(&self) -> Option<ffmpeg::Stream> {
        self.format_context
            .streams()
            .best(ffmpeg::media::Type::Audio)
    }

    /// フォーマットコンテクストへの参照を取得
    pub fn format_context(&self) -> &ffmpeg::format::context::Input {
        &self.format_context
    }

    /// フォーマットコンテクストの可変参照を取得
    pub fn format_context_mut(&mut self) -> &mut ffmpeg::format::context::Input {
        &mut self.format_context
    }
}