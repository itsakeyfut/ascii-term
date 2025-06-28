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
    fn determine_media_type(info: &MediaInfo) -> MediaType {
        match (info.has_video, info.has_audio) {
            (true, _) => MediaType::Video,
            (false, true) => MediaType::Audio,
            (false, false) => MediaType::Unknown,
        }
    }
}