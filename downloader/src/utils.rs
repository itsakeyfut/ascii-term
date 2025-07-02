use std::io::Write;
use std::path::Path;

use reqwest;
use tempfile::NamedTempFile;
use url::Url;

use crate::errors::{DownloaderError, Result};

/// URL検証ユーティリティ
pub struct UrlValidator;

impl UrlValidator {
    /// URLが有効化どうかチェック
    pub fn is_valid_url(url: &str) -> bool {
        Url::parse(url).is_ok()
    }
}