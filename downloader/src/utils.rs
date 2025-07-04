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
            return Err(DownloaderError::UnsupportedUrl(format!("Invalid URL: {}", url)));
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

    /// プログレスコードバック付きでダウンロード
    pub async fn download_with_progress<F>(
        url: &str,
        mut progress_callback: F,
    ) -> Result<NamedTempFile>
    where
        F: FnMut(u64, Option<u64>),
    {
        if !UrlValidator::is_http_url(url) {
            return Err(DownloaderError::UnsupportedUrl(format!("Invalid URL: {}", url)));
        }

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| DownloaderError::Network(e))?;

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

        let total_size = response.content_length();
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| DownloaderError::Io(e))?;

        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| DownloaderError::Network(e))?;
            temp_file
                .write_all(&chunk)
                .map_err(|e| DownloaderError::Io(e))?;
            
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }

        temp_file
            .flush()
            .map_err(|e| DownloaderError::Io(e))?;

        Ok(temp_file)
    }

    /// ファイルサイズを取得（ダウンロードせずに）
    pub async fn get_file_size(url: &str) -> Result<Option<u64>> {
        if !UrlValidator::is_http_url(url) {
            return Err(DownloaderError::UnsupportedUrl(format!("Invalid URL: {}", url)));
        }

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| DownloaderError::Network(e))?;

        let response = client
            .head(url)
            .send()
            .await
            .map_err(|e| DownloaderError::Network(e))?;

        if response.status().is_success() {
            Ok(response.content_length())
        } else {
            Err(DownloaderError::Download(format!(
                "HTTP error: {} for URL: {}",
                response.status(),
                url
            )))
        }
    }
}

/// ファイル名生成ユーティリティ
pub struct FileNameGenerator;

impl FileNameGenerator {
    /// URLから適切なファイル名を生成
    pub fn generate_from_url(url: &str, default_extension: Option<&str>) -> String {
        if let Ok(parsed_url) = Url::parse(url) {
            // URLのパスからファイル名を抽出
            if let Some(segments) = parsed_url.path_segments() {
                if let Some(filename) = segments.last() {
                    if !filename.is_empty() && filename.contains('.') {
                        return Self::sanitize_filename(filename);
                    }
                }
            }
        }

        // URLからファイル名を抽出できない場合
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Some(ext) = default_extension {
            format!("download_{}.{}", timestamp, ext)
        } else {
            format!("download_{}", timestamp)
        }
    }

    /// ファイル名をサニタイズ（安全な文字のみ）
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// 一意なファイル名を生成
    pub fn generate_unique<P: AsRef<Path>>(base_path: P, filename: &str) -> String {
        let base_path = base_path.as_ref();
        let mut candidate = filename.to_string();
        let mut cnt = 1;

        while base_path.join(&candidate).exists() {
            if let Some(pos) = filename.rfind('.') {
                let (name, ext) = filename.split_at(pos);
                candidate = format!("{}_{}{}", name, cnt, ext);
            } else {
                candidate = format!("{}_{}", filename, cnt);
            }
            cnt += 1;
        }

        candidate
    }
}

/// メディアタイプ検出ユーティリティ
pub struct MediaTypeDetector;

impl MediaTypeDetector {
    /// URLからメディアタイプを推測
    pub fn detect_from_url(url: &str) -> MediaType {
        if UrlValidator::is_youtube_url(url) {
            return MediaType::Video;
        }

        if UrlValidator::is_streaming_site(url) {
            return MediaType::Video;
        }

        if UrlValidator::is_direct_media_url(url) {
            return Self::detect_from_extension(url);
        }

        MediaType::Unknown
    }

    /// ファイル拡張子からメディアタイプを検出
    fn detect_from_extension(url: &str) -> MediaType {
        if let Ok(parsed) = Url::parse(url) {
            if let Some(path) = parsed.path_segments() {
                if let Some(last_segment) = path.last() {
                    if let Some(ext) = last_segment.split('.').last() {
                        return Self::classify_extension(ext);
                    }
                }
            }
        }

        MediaType::Unknown
    }

    /// 拡張子を分類
    fn classify_extension(ext: &str) -> MediaType {
        match ext.to_lowercase().as_str() {
            // 動画
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" |
            "3gp" | "ogv" | "ts" | "mts" | "m2ts" => MediaType::Video,
            
            // 音声
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" => MediaType::Audio,
            
            // 画像
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "svg" => MediaType::Image,
            
            _ => MediaType::Unknown,
        }
    }
}

/// メディアタイプ列挙型
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Video,
    Audio,
    Image,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validator() {
        assert!(UrlValidator::is_valid_url("https://example.com/video.mp4"));
        assert!(UrlValidator::is_http_url("https://example.com"));
        assert!(!UrlValidator::is_http_url("ftp://example.com"));
        
        assert!(UrlValidator::is_youtube_url("https://www.youtube.com/watch?v=123"));
        assert!(UrlValidator::is_youtube_url("https://youtu.be/123"));
        assert!(!UrlValidator::is_youtube_url("https://example.com"));
        
        assert!(UrlValidator::is_direct_media_url("https://example.com/video.mp4"));
        assert!(!UrlValidator::is_direct_media_url("https://example.com/page.html"));
    }

    #[test]
    fn test_filename_generator() {
        let filename = FileNameGenerator::generate_from_url(
            "https://example.com/test.mp4", 
            Some("mp4")
        );
        assert!(filename.contains("test.mp4") || filename.contains("download_"));

        let sanitized = FileNameGenerator::sanitize_filename("test<>file.mp4");
        assert_eq!(sanitized, "test__file.mp4");
    }

    #[test]
    fn test_media_type_detector() {
        assert_eq!(
            MediaTypeDetector::detect_from_url("https://www.youtube.com/watch?v=123"),
            MediaType::Video
        );
        assert_eq!(
            MediaTypeDetector::detect_from_url("https://example.com/song.mp3"),
            MediaType::Audio
        );
        assert_eq!(
            MediaTypeDetector::detect_from_url("https://example.com/image.jpg"),
            MediaType::Image
        );
    }
}