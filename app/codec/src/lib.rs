pub mod audio;
pub mod errors;
pub mod image;
pub mod media;
pub mod video;

pub use errors::{MediaError, Result};
pub use media::{MediaFile, MediaInfo, MediaType};

/// ライブラリの初期化
pub fn init() -> Result<()> {
    Ok(())
}

/// デバッグ関数：動画の基本情報を表示
pub fn test_media_info<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    let path_str = path
        .as_ref()
        .to_str()
        .ok_or_else(|| MediaError::InvalidFormat("Invalid path".to_string()))?;

    let info = avio::open(path_str)?;

    println!("Media Info:");
    println!("  Has video: {}", info.has_video());
    println!("  Has audio: {}", info.has_audio());
    println!("  Duration: {:?}", info.duration());
    if let Some(fps) = info.frame_rate() {
        println!("  FPS: {:.2}", fps);
    }
    if let Some((width, height)) = info.resolution() {
        println!("  Video: {}x{}", width, height);
    }
    if let Some(sample_rate) = info.sample_rate() {
        println!("  Sample rate: {} Hz", sample_rate);
    }
    if let Some(channels) = info.channels() {
        println!("  Channels: {}", channels);
    }

    Ok(())
}

/// 最初のフレームを取得
pub fn test_first_frame<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    use crate::video::VideoDecoder;

    let path_str = path
        .as_ref()
        .to_str()
        .ok_or_else(|| MediaError::InvalidFormat("Invalid path".to_string()))?;

    let mut decoder = VideoDecoder::new(path_str, 0, 0)?;

    if let Some(frame) = decoder.decode_one()? {
        println!("First frame decoded successfully!");
        println!("  Size: {}x{}", frame.width, frame.height);
        println!("  Format: {:?}", frame.format);
        println!("  Timestamp: {:?}", frame.timestamp);
        println!("  Data size: {} bytes", frame.data.len());
    } else {
        println!("No frame decoded");
    }

    Ok(())
}
