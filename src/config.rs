use serde::Deserialize;
use shikumi::{ConfigDiscovery, Format};
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
            api_url: "https://api.cursor.com".into(),
            api_key_file: PathBuf::from(&home).join(".config/cursor/api-key"),
            default_model: "claude-4.6-opus-high-thinking".into(),
            output: OutputFormat::Pretty,
            poll_interval: 5,
        }
    }
}

impl KurageConfig {
    pub fn load() -> Self {
        // Priority:
        // 1. KURAGE_CONFIG env (set by Nix HM module for MCP server context)
        // 2. shikumi standard discovery (XDG_CONFIG_HOME, ~/.config/kurage/)
        // 3. Defaults

        if let Ok(path) = std::env::var("KURAGE_CONFIG") {
            let path = PathBuf::from(path);
            if path.exists()
                && let Ok(content) = std::fs::read_to_string(&path)
            {
                match serde_yaml_ng::from_str::<Self>(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        tracing::warn!("failed to parse {}: {e}", path.display());
                    }
                }
            }
        }

        let discovery = ConfigDiscovery::new("kurage")
            .env_override("KURAGE_CONFIG")
            .formats(&[Format::Yaml]);

        if let Ok(path) = discovery.discover()
            && let Ok(content) = std::fs::read_to_string(&path)
        {
            match serde_yaml_ng::from_str::<Self>(&content) {
                Ok(config) => return config,
                Err(e) => {
                    tracing::warn!("failed to parse {}: {e}", path.display());
                }
            }
        }

        Self::default()
    }
}
