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
        // 2. XDG_CONFIG_HOME/kurage/kurage.yaml
        // 3. ~/.config/kurage/kurage.yaml
        // 4. Defaults

        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());

        let candidates: Vec<PathBuf> = [
            // Nix module sets this for MCP server processes that lack user env
            std::env::var("KURAGE_CONFIG").map(PathBuf::from).ok(),
            std::env::var("XDG_CONFIG_HOME")
                .map(|x| PathBuf::from(x).join("kurage/kurage.yaml"))
                .ok(),
            Some(PathBuf::from(&home).join(".config/kurage/kurage.yaml")),
        ]
        .into_iter()
        .flatten()
        .collect();

        for candidate in &candidates {
            if candidate.exists() {
                if let Ok(content) = std::fs::read_to_string(candidate) {
                    // serde_yaml_ng parses both YAML and JSON (YAML is a superset)
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
