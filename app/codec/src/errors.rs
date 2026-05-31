use thiserror::Error;

pub type Result<T> = std::result::Result<T, MediaError>;

#[derive(Error, Debug)]
pub enum MediaError {
    #[error("Decode error: {0}")]
    Decode(#[from] avio::DecodeError),

    #[error("Probe error: {0}")]
    Probe(#[from] avio::ProbeError),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Video error: {0}")]
    Video(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported codec: {0}")]
    UnsupportedCodec(String),

    #[error("Pipeline error: {0}")]
    Pipeline(String),
}
