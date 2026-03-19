#![allow(dead_code)]

mod errors;
mod utils;
mod youtube;

pub use errors::{DownloaderError, Result};
pub use youtube::download_video;
