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
