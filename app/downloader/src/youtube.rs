use std::path::PathBuf;

use tempfile::NamedTempFile;
use tokio::process::Command;

use crate::errors::{DownloaderError, Result};

/// Download YouTube video
pub async fn download_video(url: &str, _browser: &str) -> Result<PathBuf> {
    check_ytdlp_installed().await?;

    let temp_file = NamedTempFile::new().map_err(DownloaderError::Io)?;
    let temp_path = temp_file.path().to_path_buf();
    let temp_path_str = temp_path
        .to_str()
        .ok_or_else(|| DownloaderError::Process("Temporary path is not valid UTF-8".to_string()))?;

    run_ytdlp(
        &[url, "-f", "best[ext=mp4]/best", "-o", temp_path_str],
        "yt-dlp failed",
    )
    .await?;

    let persistent_path = temp_file.into_temp_path();
    Ok(persistent_path.to_path_buf())
}

/// Get video information (metadata only)
pub async fn get_video_info(url: &str) -> Result<VideoInfo> {
    check_ytdlp_installed().await?;

    let stdout = run_ytdlp(
        &[url, "--dump-json", "--no-download"],
        "Failed to get video info",
    )
    .await?;
    parse_json(&stdout, "video info")
}

/// Get available formats
pub async fn list_formats(url: &str) -> Result<Vec<FormatInfo>> {
    check_ytdlp_installed().await?;

    let stdout = run_ytdlp(
        &[url, "--list-formats", "--dump-json"],
        "Failed to list formats",
    )
    .await?;
    parse_json(&stdout, "formats")
}

/// Check if yt-dlp is installed
async fn check_ytdlp_installed() -> Result<()> {
    let output = Command::new("yt-dlp").arg("--version").output().await;

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(DownloaderError::DependencyMissing(
            "yt-dlp is not installed. Please install it from https://github.com/yt-dlp/yt-dlp"
                .to_string(),
        )),
    }
}

/// Run yt-dlp with the given arguments, returning captured stdout on success.
async fn run_ytdlp(args: &[&str], failure_context: &str) -> Result<Vec<u8>> {
    let output = Command::new("yt-dlp")
        .args(args)
        .output()
        .await
        .map_err(|e| DownloaderError::Process(format!("Failed to execute yt-dlp: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(DownloaderError::Download(format!(
            "{}: {}",
            failure_context, error_msg
        )));
    }

    Ok(output.stdout)
}

/// Parse yt-dlp JSON output into the requested type.
fn parse_json<T: serde::de::DeserializeOwned>(stdout: &[u8], what: &str) -> Result<T> {
    let json_str = String::from_utf8_lossy(stdout);
    serde_json::from_str(&json_str)
        .map_err(|e| DownloaderError::Parse(format!("Failed to parse {}: {}", what, e)))
}

/// Video information structure
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

/// Format information structure
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
    pub tbr: Option<f64>, // Total bitrate
    pub vbr: Option<f64>, // Video bitrate
    pub abr: Option<f64>, // Audio bitrate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ytdlp_check() {
        // Test yt-dlp availability check
        // Should be properly skipped in actual environment
        let result = check_ytdlp_installed().await;
        match result {
            Ok(()) => println!("yt-dlp is available"),
            Err(e) => println!("yt-dlp check failed: {}", e),
        }
    }
}
