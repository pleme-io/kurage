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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = KurageConfig::default();
        assert_eq!(config.api_url, "https://api.cursor.com");
        assert_eq!(config.default_model, "claude-4.6-opus-high-thinking");
        assert_eq!(config.output, OutputFormat::Pretty);
        assert_eq!(config.poll_interval, 5);
        assert!(config
            .api_key_file
            .to_string_lossy()
            .ends_with(".config/cursor/api-key"));
    }

    #[test]
    fn deserialize_full_yaml() {
        let yaml = r"
api_url: https://custom.example.com
api_key_file: /etc/cursor/key
default_model: gpt-4o
output: json
poll_interval: 10
";
        let config: KurageConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.api_url, "https://custom.example.com");
        assert_eq!(config.api_key_file, PathBuf::from("/etc/cursor/key"));
        assert_eq!(config.default_model, "gpt-4o");
        assert_eq!(config.output, OutputFormat::Json);
        assert_eq!(config.poll_interval, 10);
    }

    #[test]
    fn deserialize_partial_yaml_uses_defaults() {
        let yaml = "output: table\n";
        let config: KurageConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.output, OutputFormat::Table);
        assert_eq!(config.api_url, "https://api.cursor.com");
        assert_eq!(config.poll_interval, 5);
    }

    #[test]
    fn deserialize_empty_yaml_is_default() {
        let yaml = "{}";
        let config: KurageConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.api_url, "https://api.cursor.com");
        assert_eq!(config.output, OutputFormat::Pretty);
    }

    #[test]
    fn output_format_all_variants() {
        for (s, expected) in [
            ("\"json\"", OutputFormat::Json),
            ("\"pretty\"", OutputFormat::Pretty),
            ("\"table\"", OutputFormat::Table),
        ] {
            let fmt: OutputFormat = serde_json::from_str(s).unwrap();
            assert_eq!(fmt, expected);
        }
    }

    #[test]
    fn load_from_kurage_config_env() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(
            &mut tmp,
            b"api_url: https://test.example.com\npoll_interval: 99\n",
        )
        .unwrap();
        // SAFETY: test binary is single-threaded for this env var
        unsafe { std::env::set_var("KURAGE_CONFIG", tmp.path().as_os_str()) };
        let config = KurageConfig::load();
        unsafe { std::env::remove_var("KURAGE_CONFIG") };
        assert_eq!(config.api_url, "https://test.example.com");
        assert_eq!(config.poll_interval, 99);
    }

    #[test]
    fn load_falls_back_to_default() {
        // SAFETY: test binary is single-threaded for this env var
        unsafe { std::env::set_var("KURAGE_CONFIG", "/dev/null") };
        let config = KurageConfig::load();
        unsafe { std::env::remove_var("KURAGE_CONFIG") };
        assert_eq!(config.api_url, "https://api.cursor.com");
    }
}
