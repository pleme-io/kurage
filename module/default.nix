# Kurage home-manager module — MCP server + CLI for Cursor Cloud Agents
#
# Cross-platform: Darwin + Linux (MCP server entry is platform-agnostic)
#
# Namespace: services.kurage.*
#
# Provides:
#   - MCP server entry (consumed by anvil/claude modules for all AI agents)
#   - CLI binary in PATH
#   - Optional config file generation (~/.config/kurage/kurage.yaml)
#
# Usage:
#   services.kurage.enable = true;
#   services.kurage.mcp.enable = true;
#   services.kurage.settings = {
#     default_model = "claude-opus-4-6";
#     output = "pretty";
#   };
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

  # Generate YAML config from Nix options
  configYaml = pkgs.writeText "kurage.yaml"
    (builtins.toJSON ({
      api_url = cfg.settings.apiUrl;
      default_model = cfg.settings.defaultModel;
      output = cfg.settings.output;
      poll_interval = cfg.settings.pollInterval;
    } // optionalAttrs (cfg.settings.apiKeyFile != null) {
      api_key_file = cfg.settings.apiKeyFile;
    }));
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
          When null, kurage falls back to CURSOR_API_KEY env var
          or ~/.config/cursor/api-key.
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
    };
  };

  config = mkMerge [
    # ── CLI binary + config file (all platforms) ─────────────────────
    (mkIf cfg.enable {
      home.packages = [ cfg.package ];

      xdg.configFile."kurage/kurage.yaml".source = configYaml;
    })

    # ── MCP server entry (all platforms, consumed by anvil) ──────────
    (mkIf mcpCfg.enable {
      services.kurage.mcp.serverEntry = mkMcpServerEntry {
        command = "${mcpCfg.package}/bin/kurage";
      };
    })
  ];
}
