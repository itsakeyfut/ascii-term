pub mod errors;
pub mod media;
pub mod video;
pub mod audio;
pub mod image;
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

/// デバッグ関数：動画の基本情報を表示
pub fn test_media_info<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    let media = MediaFile::open(path)?;
    
    println!("Media Info:");
    println!("  Type: {:?}", media.media_type);
    println!("  Duration: {:?}", media.info.duration);
    if let Some(fps) = media.info.fps {
        println!("  FPS: {:.2}", fps);
    }
    if let (Some(width), Some(height)) = (media.info.width, media.info.height) {
        println!("  Video: {}x{}", width, height);
    }
    if let Some(codec) = &media.info.video_codec {
        println!("  Video Codec: {}", codec);
    }
    if let (Some(channels), Some(sample_rate)) = (media.info.channels, media.info.sample_rate) {
        println!("  Audio: {} channels, {} Hz", channels, sample_rate);
    }
    if let Some(codec) = &media.info.audio_codec {
        println!("  Audio Codec: {}", codec);
    }
    
    Ok(())
}

/// 最初のフレームを取得
pub fn test_first_frame<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    use crate::video::VideoDecoder;
    
    let mut media = MediaFile::open(path)?;
    let mut decoder = VideoDecoder::new(&media)?;
    
    // 最初のパケットを読み込み
    let (stream, packet) = media.read_packet()?;
    
    // フレームをデコード
    if let Some(frame) = decoder.decode_next_frame(&packet)? {
        println!("First frame decoded successfully!");
        println!("  Size: {}x{}", frame.width, frame.height);
        println!("  Format: {:?}", frame.format);
        println!("  Timestamp: {:?}", frame.timestamp);
        println!("  Data size: {} bytes", frame.data.len());
    } else {
        println!("No frame decoded from first packet");
    }
    
    Ok(())
}