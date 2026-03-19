use std::path::Path;
use std::time::Duration;

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
#[derive(Debug, Clone)]
pub struct MediaFile {
    pub path: String,
    pub media_type: MediaType,
    pub info: MediaInfo,
}

impl MediaFile {
    /// ファイルパスからメディアファイルを開く
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| MediaError::InvalidFormat("Invalid path".to_string()))?
            .to_string();

        let avio_info = avio::open(&path_str)?;

        let info = MediaInfo {
            duration: Some(avio_info.duration()),
            width: avio_info.resolution().map(|(w, _)| w),
            height: avio_info.resolution().map(|(_, h)| h),
            fps: avio_info.frame_rate(),
            has_video: avio_info.has_video(),
            has_audio: avio_info.has_audio(),
            video_codec: avio_info
                .primary_video()
                .map(|v| format!("{:?}", v)),
            audio_codec: avio_info
                .primary_audio()
                .map(|a| format!("{:?}", a)),
            sample_rate: avio_info.sample_rate(),
            channels: avio_info.channels().map(|c| c as u16),
        };

        let media_type = Self::determine_media_type(&info);

        Ok(MediaFile {
            path: path_str,
            media_type,
            info,
        })
    }

    /// メディアタイプを判定
    fn determine_media_type(info: &MediaInfo) -> MediaType {
        if info.has_video {
            MediaType::Video
        } else if info.has_audio {
            MediaType::Audio
        } else {
            MediaType::Unknown
        }
    }
}

/// 静的画像ファイルを検出
pub fn is_image_file<P: AsRef<Path>>(path: P) -> bool {
    if let Some(ext) = path.as_ref().extension() {
        if let Some(ext_str) = ext.to_str() {
            matches!(
                ext_str.to_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "bmp" | "gif" | "webp" | "tiff" | "tif"
            )
        } else {
            false
        }
    } else {
        false
    }
}
