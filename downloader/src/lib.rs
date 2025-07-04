mod errors;
mod youtube;
mod utils;

pub use youtube::download_video;
pub use errors::{DownloaderError, Result};