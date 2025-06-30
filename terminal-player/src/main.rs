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

    Ok(())
}