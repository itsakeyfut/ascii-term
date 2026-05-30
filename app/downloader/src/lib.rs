mod errors;
mod youtube;

pub use errors::{DownloaderError, Result};
pub use youtube::{FormatInfo, VideoInfo, download_video, get_video_info, list_formats};
