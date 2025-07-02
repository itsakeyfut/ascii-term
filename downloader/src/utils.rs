use std::io::Write;
use std::path::Path;

use reqwest;
use tempfile::NamedTempFile;
use url::Url;

use crate::errors::{DownloaderError, Result};

/// URL検証ユーティリティ
pub struct UrlValidator;

impl UrlValidator {
    /// URLが有効化どうかチェック
    pub fn is_valid_url(url: &str) -> bool {
        Url::parse(url).is_ok()
    }

    /// HTTPまたはHTTPS URLかチェック
    pub fn is_http_url(url: &str) -> bool {
        if let Ok(parsed) = Url::parse(url) {
            matches!(parsed.scheme(), "http" | "https")
        } else {
            false
        }
    }

    /// YouTube URLかチェック
    pub fn is_youtube_url(url: &str) -> bool {
        if let Ok(parsed) = Url::parse(url) {
            if let Some(domain) = parsed.domain() {
                return domain.contains("youtube.com")
                    || domain.contains("youtu.be")
                    || domain.contains("youtube-nocookie.com");
            }
        }
        false
    }

    /// 動画ストリーミングサイトかチェック
    pub fn is_streaming_site(url: &str) -> bool {
        if let Ok(parsed) = Url::parse(url) {
            if let Some(domain) = parsed.domain() {
                let streaming_domains = [
                    "youtube.com", "youtu.be", "twitch.tv", "tiktok.com",
                    "facebook.com", "instagram.com", "x.com", "reddit.com",
                    "dailymotion.com"
                ];

                return streaming_domains.iter().any(|&d| domain.contains(d));
            }
        }
        false
    }
}