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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_api_key_error_display() {
        let err = KurageError::NoApiKey {
            path: PathBuf::from("/home/user/.config/cursor/api-key"),
        };
        let msg = err.to_string();
        assert!(msg.contains("API key not found"));
        assert!(msg.contains("/home/user/.config/cursor/api-key"));
        assert!(msg.contains("CURSOR_API_KEY"));
    }

    #[test]
    fn no_api_key_debug() {
        let err = KurageError::NoApiKey {
            path: PathBuf::from("/tmp/key"),
        };
        let debug = format!("{err:?}");
        assert!(debug.contains("NoApiKey"));
    }
}
