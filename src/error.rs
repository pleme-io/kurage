use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum KurageError {
    #[error("HTTP error: {0}")]
    Http(#[from] todoku::TodokuError),

    #[error("API key not found — set --api-key, CURSOR_API_KEY, or create {path}")]
    NoApiKey { path: PathBuf },
}

pub type Result<T> = std::result::Result<T, KurageError>;
