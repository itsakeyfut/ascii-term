use thiserror::Error;

pub type Result<T> = std::result::Result<T, DownloaderError>;

#[derive(Error, Debug)]
pub enum DownloaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Parse(String),

    #[error("Download failed: {0}")]
    Download(String),

    #[error("Process execution error: {0}")]
    Process(String),

    #[error("Dependency missing: {0}")]
    DependencyMissing(String),
}
