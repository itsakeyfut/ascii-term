use thiserror::Error;

pub type Result<T> = std::result::Result<T, DownloaderError>;

#[derive(Error, Debug)]
pub enum DownloaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("JSON parsing error: {0}")]
    Parse(String),

    #[error("Download failed: {0}")]
    Download(String),

    #[error("Process execution error: {0}")]
    Process(String),

    #[error("Dependency missing: {0}")]
    DependencyMissing(String),

    #[error("Unsupported URL: {0}")]
    UnsupportedUrl(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
