# Kurage home-manager module — MCP server + CLI for Cursor Cloud Agents
#
# Cross-platform: Darwin + Linux (MCP server entry is platform-agnostic)
#
# Namespace: services.kurage.*
#
# Provides:
#   - MCP server entry (consumed by claude/anvil for all AI agents)
#   - CLI binary in PATH
#   - Config file generation (~/.config/kurage/kurage.yaml)
#   - Env propagation: CURSOR_API_KEY passed to MCP server process
#
# Two integration paths (use one or both):
#   1. Service-level: services.kurage.mcp.enable + claude.mcp.kurage.enable
#      → Claude reads config.services.kurage.mcp.serverEntry
#   2. Anvil-level: define kurage in anvil.mcp.servers with envFiles
#      → All agents (Cursor, Claude, OpenCode) get kurage
#
# Usage:
#   services.kurage.enable = true;
#   services.kurage.mcp.enable = true;
#   services.kurage.settings.apiKeyFile = "~/.config/cursor/api-key";
#
# Module factory: receives { hmHelpers } from flake.nix, returns HM module.
{ hmHelpers }:
{
  lib,
  config,
  pkgs,
  ...
}:
with lib; let
  inherit (hmHelpers) mkMcpOptions mkMcpServerEntry;
  cfg = config.services.kurage;
  mcpCfg = cfg.mcp;
  homeDir = config.home.homeDirectory;

  # Default API key file path (sops-nix decrypted location)
  defaultApiKeyFile = "${homeDir}/.config/cursor/api-key";

  # Resolved API key file (explicit setting > default)
  resolvedApiKeyFile =
    if cfg.settings.apiKeyFile != null
    then cfg.settings.apiKeyFile
    else defaultApiKeyFile;

  # Config file (JSON is valid YAML — serde_yaml_ng parses both)
  configFile = pkgs.writeText "kurage.yaml"
    (builtins.toJSON ({
      api_url = cfg.settings.apiUrl;
      api_key_file = resolvedApiKeyFile;
      default_model = cfg.settings.defaultModel;
      output = cfg.settings.output;
      poll_interval = cfg.settings.pollInterval;
    }));

  # MCP server environment — ensures CURSOR_API_KEY is available to the
  # MCP server process. This is critical: when Claude Code launches kurage
  # as an MCP server, the process inherits NO user env. Without this, the
  # API key file fallback in auth.rs is the only auth path.
  mcpEnv = optionalAttrs cfg.settings.propagateApiKey {
    KURAGE_CONFIG = "${configFile}";
  };
in {
  options.services.kurage = {
    enable = mkEnableOption "kurage — Cursor Cloud Agents CLI + MCP server";

    package = mkOption {
      type = types.package;
      default = pkgs.kurage or (throw "kurage package not found — set services.kurage.package");
      defaultText = literalExpression "pkgs.kurage";
      description = "The kurage binary package.";
    };

    # MCP server options (from substrate helpers)
    mcp = mkMcpOptions {
      defaultPackage = pkgs.kurage or (throw "kurage package not found");
    };

    # Declarative config options
    settings = {
      apiUrl = mkOption {
        type = types.str;
        default = "https://api.cursor.com/v0";
        description = "Cursor Cloud Agents API base URL.";
      };

      apiKeyFile = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = ''
          Path to file containing the Cursor API key.
          When null, defaults to ~/.config/cursor/api-key
          (standard sops-nix decrypted location).
        '';
      };

      defaultModel = mkOption {
        type = types.str;
        default = "claude-opus-4-6";
        description = "Default AI model for cloud agents.";
      };

      output = mkOption {
        type = types.enum [ "json" "pretty" "table" ];
        default = "pretty";
        description = "Default output format for CLI.";
      };

      pollInterval = mkOption {
        type = types.ints.between 1 300;
        default = 5;
        description = "Polling interval in seconds for --follow commands.";
      };

      propagateApiKey = mkOption {
        type = types.bool;
        default = true;
        description = ''
          Pass config file path to the MCP server process via KURAGE_CONFIG env.
          Ensures the MCP server can find the API key when launched by Claude
          Code or other MCP clients that don't inherit user environment.
        '';
      };
    };
  };

  config = mkMerge [
    # ── CLI binary + config file (all platforms) ─────────────────────
    (mkIf cfg.enable {
      home.packages = [ cfg.package ];

      xdg.configFile."kurage/kurage.yaml".source = configFile;
    })

    # ── MCP server entry (all platforms, consumed by claude/anvil) ────
    (mkIf mcpCfg.enable {
      services.kurage.mcp.serverEntry = mkMcpServerEntry ({
        command = "${mcpCfg.package}/bin/kurage";
      } // optionalAttrs (mcpEnv != {}) {
        env = mcpEnv;
      });
    })
  ];
}
