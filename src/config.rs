use serde::Deserialize;
use shikumi::{ConfigDiscovery, Format};
use std::path::PathBuf;

/// Kurage configuration loaded from YAML config or defaults.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct KurageConfig {
    /// Base URL for the Cursor Cloud Agents API.
    pub api_url: String,
    /// Path to file containing the Cursor API key.
    pub api_key_file: PathBuf,
    /// Default model ID to use when launching agents.
    pub default_model: String,
    /// CLI output format.
    pub output: OutputFormat,
    /// Seconds between polls for `--follow` modes.
    pub poll_interval: u64,
}

/// CLI output format selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Emit raw JSON.
    Json,
    /// Human-readable formatted text.
    Pretty,
    /// Tabular output using `comfy-table`.
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
