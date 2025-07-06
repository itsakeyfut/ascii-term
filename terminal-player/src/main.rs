mod audio;
mod char_maps;
mod player;
mod renderer;
mod terminal;

use anyhow::Result;
use clap::Parser;

use media_core::MediaFile;

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

    /// Browser for cookie extraction (for YouTube)
    #[arg(short, long, default_value = "firefox")]
    browser: String,

    /// Loop playback
    #[arg(short, long)]
    loop_playback: bool,

    /// Character map selection (0-9)
    #[arg(short, long, default_value = "0")]
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

    /// Diagnose audio system
    #[arg(long)]
    diagnose_audio: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // 音声診断モード
    if args.diagnose_audio {
        println!("Running audio system diagnostics...");
        return audio::diagnose_audio_system();
    }

    // 初期化
    media_core::init()?;

    // 入力の処理
    let media_path = if is_url(&args.input) {
        handle_url_input(&args.input, &args.browser).await?
    } else {
        args.input.clone()
    };

    // メディアファイルを開く
    let media_file = MediaFile::open(&media_path)?;

    println!("Media Info:");
    println!("  Type: {:?}", media_file.media_type);
    println!("  Duration: {:?}", media_file.info.duration);
    if let Some(fps) = media_file.info.fps {
        println!("  FPS: {:.2}", fps);
    }
    if media_file.info.has_video {
        println!(
            "  Video: {}x{}",
            media_file.info.width.unwrap_or(0),
            media_file.info.height.unwrap_or(0)
        );
        if let Some(codec) = &media_file.info.video_codec {
            println!("  Video Codec: {}", codec);
        }
    }
    if media_file.info.has_audio {
        println!(
            "  Audio: {} channels, {} Hz",
            media_file.info.channels.unwrap_or(0),
            media_file.info.sample_rate.unwrap_or(0)
        );
        if let Some(codec) = &media_file.info.audio_codec {
            println!("  Audio Codec: {}", codec);
        }
    }

    // 音声再生の設定
    let enable_audio = !args.no_audio && media_file.info.has_audio;

    if enable_audio {
        println!("Audio playback enabled");
        // 音声システムの簡易チェック
        if let Err(e) = audio::diagnose_audio_system() {
            eprintln!("Warning: Audio system check failed: {}", e);
            eprintln!("Continuing with audio disabled...");
        }
    } else {
        println!("Audio playback disabled");
    }

    // プレイヤー設定
    let config = player::PlayerConfig {
        fps: args.fps,
        loop_playback: args.loop_playback,
        char_map_index: args.char_map,
        grayscale: args.gray,
        width_modifier: args.width_mod,
        allow_frame_skip: args.allow_frame_skip,
        add_newlines: args.newlines,
        enable_audio: !args.no_audio && media_file.info.has_audio,
    };

    // プレイヤーを作成して実行
    let mut player = player::Player::new(media_file, config)?;
    player.run().await?;

    Ok(())
}

fn is_url(input: &str) -> bool {
    input.starts_with("http://") || input.starts_with("https://")
}

async fn handle_url_input(url: &str, browser: &str) -> Result<String> {
    use url::Url;

    let parsed_url = Url::parse(url)?;

    if let Some(domain) = parsed_url.domain() {
        if domain.contains("youtube.com") || domain.contains("youtu.be") {
            println!("Downloading YouTube video...");
            let temp_path = downloader::download_video(url, browser).await?;
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
