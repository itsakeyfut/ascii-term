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

    /// メディアファイルの直接リンクかチェック
    pub fn is_direct_media_url(url: &str) -> bool {
        if let Ok(parsed) = Url::parse(url) {
            if let Some(path) = parsed.path_segments() {
                if let Some(last_segment) = path.last() {
                    let media_extensions = [
                        // 動画
                        "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v",
                        "3gp", "ogv", "ts", "mts", "m2ts",
                        // 音声
                        "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a", "opus",
                        // 画像
                        "jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "svg"
                    ];
                    
                    if let Some(ext) = last_segment.split('.').last() {
                        return media_extensions.contains(&ext.to_lowercase().as_str());
                    }
                }
            }
        }
        false
    }
}

/// ファイルダウンロードユーティリティ
pub struct FileDownloader;

impl FileDownloader {
    /// URLからファイルをダウンロード
    pub async fn download_to_temp(url: &str) -> Result<NamedTempFile> {
        // URL検証
        if !UrlValidator::is_http_url(url) {
            return Err(DownloaderError::Unsupported(format!("Invalid URL: {}", url)));
        }

        // HTTPクライアントを作成
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| DownloaderError::Network(e))?;

        // ファイルをダウンロード
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| DownloaderError::Network(e))?;

        if !response.status().is_success() {
            return Err(DownloaderError::Download(format!(
                "HTTP error: {} for URL: {}",
                response.status(),
                url
            )));
        }

        // 一時ファイルを作成
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| DownloaderError::Io(e))?;

        // レスポンスボディを一時ファイルに書き込み
        let content = response
            .bytes()
            .await
            .map_err(|e| DownloaderError::Network(e))?;

        temp_file
            .write_all(&content)
            .map_err(|e| DownloaderError::Io(e))?;

        temp_file
            .flush()
            .map_err(|e| DownloaderError::Io(e))?;

        Ok(temp_file)
    }

    /// URLからファイルを指定パスにダウンロード
    pub async fn download_to_path<P: AsRef<Path>>(url: &str, path: P) -> Result<()> {
        let temp_file = Self::download_to_temp(url).await?;
        
        // 一時ファイルを指定パスにコピー
        std::fs::copy(temp_file.path(), &path)
            .map_err(|e| DownloaderError::Io(e))?;

        Ok(())
    }
}