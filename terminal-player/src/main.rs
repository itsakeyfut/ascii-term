mod char_maps;
mod renderer;
mod player;
mod terminal;
mod audio;

use std::path::Path;
use anyhow::Result;
use clap::Parser;

use media_core::{MediaFile, MediaType};

#[derive(Parser, Debug)]
#[command(name = "ascii_term")]
#[command(about = "Terminal media player with ASCII art rendering")]
struct Args {
    /// Input file path or URL
    #[arg(value_name = "INPUT")]
    input: String,

    /// Force specific frame rate
    #[arg(short, long)]
    fps: Option<f64>,

    /// Browser for cookie extraction
    #[arg(short, long, default_value = "firefox")]
    browser: String,

    /// Loop playback
    loop_playback: bool,

    /// Character map selection (0-9)
    char_map: u8,

    /// Enable grayscale mode
    #[arg(short, long)]
    gray: bool,

    /// Width modifier for character aspect ratio
    #[arg(short, long, default_value = "1")]
    width_mod: u32,

    /// Allow frame skipping when behind
    #[arg(long)]
    allow_frame_skip: bool,

    /// Add newlines to output
    #[arg(short, long)]
    newlines: bool,

    /// Disable audio playback
    #[arg(long)]
    no_audio: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    media_core::init()?;

    let media_path = if is_url(&args.input) {
        unimplemented!()
    } else {
        args.input.clone()
    };

    Ok(())
}

fn is_url(input: &str) -> bool {
    input.starts_with("http://") || input.starts_with("http://")
}

async fn handle_url_input(url: &str, browser: &str) -> Result<String> {
    use url::Url;

    let parsed_url = Url::parse(url)?;

    if let Some(domain) = parsed_url.domain() {
        if domain.contains("youtube.com") || domain.contains("youtu.be") {
            println!("Downloading YouTube video...");
            let temp_path = unimplemented!();
            return Ok(temp_path.to_string_lossy().to_string());
        }
    }

    // その他のURLの場合は直接ダウンロード
    println!("Downloading media file...");
    let temp_path = download_url(url).await?;
    Ok(temp_path)
}

async fn download_url(url: &str) -> Result<String> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let response = reqwest::get(url).await?;
    let content = response.bytes().await?;

    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(&content)?;

    let path = temp_file.into_temp_path();
    Ok(path.to_string_lossy().to_string())
}