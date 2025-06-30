pub mod errors;
pub mod media;
pub mod video;
pub mod audio;
// pub mod image;
pub mod pipeline;

pub use errors::{MediaError, Result};
pub use media::{MediaFile, MediaType, MediaInfo};
pub use pipeline::{Pipeline, PipelineBuilder};

/// ライブラリの初期化
/// 
/// 次のライブラリの初期化を行う：
/// 
/// * FFmpeg
/// * OpenCV
pub fn init() -> Result<()> {
    ffmpeg_next::init()
        .map_err(|e| MediaError::Config(format!("Failed to initialize FFmpeg: {}", e)))?;

    ffmpeg_next::log::set_level(ffmpeg_next::log::Level::Warning);

    Ok(())
}