pub mod audio;
pub mod errors;
pub mod media;
pub mod video;

pub use errors::{MediaError, Result};
pub use media::{MediaFile, MediaInfo, MediaType};

/// ライブラリの初期化
pub fn init() -> Result<()> {
    Ok(())
}
