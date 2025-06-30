use std::path::PathBuf;

use tempfile::NamedTempFile;
use tokio::process::Command;

use crate::errrors::{DownloaderError, Result};

/// YouTube動画をダウンロード
pub async fn download_video(url: &str, browser: &str) -> Result<PathBuf> {
    // yt-dlp がインストールされているかチェック
    check_ytdlp_installed().await?;

    // 一時ファイルを作成
    let temp_file = NamedTempFile::new()
        .map_err(|e| DownloaderError::Io(e))?;
    let temp_path = temp_file.path().to_path_buf();

    // yt-dlp コマンドを実行
    let output = Command::new("yt-dlp")
        .arg(url)
        .arg("--cookies-from-browser")
        .arg(browser)
        .arg("-f")
        .arg("best[ext=mp4]/best")
        .arg("-o")
        .arg(&temp_path)
        .output()
        .await
        .map_err(|e| DownloaderError::Process(format!("Failed to execute yt-dlp: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(DownloaderError::Download(format!(
            "yt-dlp failed: {}",
            error_msg
        )));
    }

    // 一時ファイルのパスを永続化
    let persistent_path = temp_file.into_temp_path();
    Ok(persistent_path.to_path_buf())
}

/// 動画情報を取得（メタデータのみ）
pub async fn get_video_info(url: &str) -> Result<VideoInfo> {
    check_ytdlp_installed().await?;

    let output = Command::new("yt-dlp")
        .arg(url)
        .arg("--dump-json")
        .arg("--no-download")
        .output()
        .await
        .map_err(|e| DownloaderError::Process(format!("Failed to execute yt-dlp: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(DownloaderError::Donwload(format!(
            "Failed to get video info: {}",
            error_msg
        )));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let info: VideoInfo = serde_json::from_str(&json_str)
        .map_err(|e| DownloaderError::Parse(format!("Failed to parse video info: {}", e)))?;

    Ok(info)
}

/// yt-dlp がインストールされているかチェック
async fn check_ytdlp_installed() -> Result<()> {
    let output = Command::new("yt-dlp")
        .arg("--version")
        .output()
        .await;

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(DownloaderError::DependencyMissing(
            "yt-dlp is not installed. Please install it from https://github.com/yt-dlp/yt-dlp".to_string()
        )),
    }
}

/// 動画情報
#[derive(Debug, serde::Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub duration: Option<f64>,
    pub uploader: Option<String>,
    pub upload_date: Option<String>,
    pub view_count: Option<i64>,
    pub like_count: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fps: Option<f64>,
    pub formats: Vec<FormatInfo>,
}

/// 形式情報
#[derive(Debug, serde::Deserialize)]
pub struct FormatInfo {
    pub format_id: String,
    pub ext: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fps: Option<f64>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub filesize: Option<i64>,
    pub tbr: Option<f64>, // 総ビットレート
    pub vbr: Option<f64>, // 動画ビットレート
    pub abr: Option<f64>, // 音声ビットレート
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ytdlp_check() {
        // yt-dlpのチェックテスト
        // 実際の環境では適切にスキップされる
        let result = check_ytdlp_installed().await;
        match result {
            Ok(()) => println!("yt-dlp is available"),
            Err(e) => println!("yt-dlp check failed: {}", e),
        }
    }
}