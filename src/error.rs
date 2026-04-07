use std::path::PathBuf;

/// Top-level error type for kurage operations.
#[derive(Debug, thiserror::Error)]
pub enum KurageError {
    /// An HTTP / network error from the underlying `todoku` client.
    #[error("HTTP error: {0}")]
    Http(#[from] todoku::TodokuError),

    /// No API key could be resolved from any source.
    #[error("API key not found — set --api-key, CURSOR_API_KEY, or create {path}")]
    NoApiKey {
        /// Path to the expected API key file.
        path: PathBuf,
    },
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, KurageError>;
