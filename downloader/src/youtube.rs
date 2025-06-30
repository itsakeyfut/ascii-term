use std::path::PathBuf;

use tempfile::NamedTempFile;
use tokio::process::Command;

use crate::errrors::{DownloaderError, Result};

/// YouTube動画をダウンロード
pub async fn download_video(url: &str, browser: &str) -> Result<PathBuf> {
    // check yt-dlp
    unimplemented!()
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