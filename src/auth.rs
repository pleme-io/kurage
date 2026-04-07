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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn config_with_key_file(path: PathBuf) -> KurageConfig {
        KurageConfig {
            api_url: "https://api.cursor.com".into(),
            api_key_file: path,
            default_model: "test".into(),
            output: crate::config::OutputFormat::Pretty,
            poll_interval: 5,
        }
    }

    #[test]
    fn resolve_explicit_key() {
        let config = config_with_key_file(PathBuf::from("/nonexistent"));
        let key = resolve_api_key(Some("explicit-key"), &config).unwrap();
        assert_eq!(key, "explicit-key");
    }

    #[test]
    fn resolve_key_from_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "  file-key-value  \n").unwrap();
        let config = config_with_key_file(tmp.path().to_path_buf());
        let key = resolve_api_key(None, &config).unwrap();
        assert_eq!(key, "file-key-value");
    }

    #[test]
    fn resolve_key_empty_file_is_error() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "   \n  ").unwrap();
        let config = config_with_key_file(tmp.path().to_path_buf());
        let result = resolve_api_key(None, &config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("API key not found"));
    }

    #[test]
    fn resolve_key_missing_file_is_error() {
        let config = config_with_key_file(PathBuf::from("/tmp/kurage-nonexistent-key-file"));
        let result = resolve_api_key(None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn expand_tilde_with_home() {
        let result = expand_tilde(Path::new("~/.config/cursor/api-key"));
        assert!(!result.to_string_lossy().starts_with("~/"));
        assert!(result.to_string_lossy().ends_with(".config/cursor/api-key"));
    }

    #[test]
    fn expand_tilde_absolute_path_unchanged() {
        let path = Path::new("/etc/kurage/key");
        let result = expand_tilde(path);
        assert_eq!(result, PathBuf::from("/etc/kurage/key"));
    }

    #[test]
    fn expand_tilde_relative_path_unchanged() {
        let path = Path::new("relative/path");
        let result = expand_tilde(path);
        assert_eq!(result, PathBuf::from("relative/path"));
    }
}
