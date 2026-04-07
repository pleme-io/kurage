use crate::config::KurageConfig;
use crate::error::{KurageError, Result};
use std::path::{Path, PathBuf};

/// Resolve the Cursor API key from (in priority order):
/// 1. Explicit CLI flag value
/// 2. `CURSOR_API_KEY` environment variable
/// 3. Contents of the configured `api_key_file`
pub fn resolve_api_key(explicit: Option<&str>, config: &KurageConfig) -> Result<String> {
    if let Some(key) = explicit {
        return Ok(key.to_string());
    }

    if let Ok(key) = std::env::var("CURSOR_API_KEY")
        && !key.is_empty()
    {
        return Ok(key);
    }

    let path = expand_tilde(&config.api_key_file);
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let key = content.trim().to_string();
            if key.is_empty() {
                Err(KurageError::NoApiKey { path })
            } else {
                Ok(key)
            }
        }
        Err(_) => Err(KurageError::NoApiKey { path }),
    }
}

fn expand_tilde(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    path.to_path_buf()
}
