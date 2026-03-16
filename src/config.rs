use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct KurageConfig {
    pub api_url: String,
    pub api_key_file: PathBuf,
    pub default_model: String,
    pub output: OutputFormat,
    pub poll_interval: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Pretty,
    Table,
}

impl Default for KurageConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        Self {
            api_url: "https://api.cursor.com/v0".into(),
            api_key_file: PathBuf::from(&home).join(".config/cursor/api-key"),
            default_model: "claude-opus-4-6".into(),
            output: OutputFormat::Pretty,
            poll_interval: 5,
        }
    }
}

impl KurageConfig {
    pub fn load() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());

        // Try XDG_CONFIG_HOME first, then ~/.config
        let candidates = [
            std::env::var("XDG_CONFIG_HOME")
                .map(|x| PathBuf::from(x).join("kurage/kurage.yaml"))
                .ok(),
            Some(PathBuf::from(&home).join(".config/kurage/kurage.yaml")),
        ];

        for candidate in candidates.into_iter().flatten() {
            if candidate.exists() {
                if let Ok(content) = std::fs::read_to_string(&candidate) {
                    match serde_yaml_ng::from_str::<Self>(&content) {
                        Ok(config) => return config,
                        Err(e) => {
                            tracing::warn!("failed to parse {}: {e}", candidate.display());
                        }
                    }
                }
            }
        }

        Self::default()
    }
}
