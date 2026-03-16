# Kurage home-manager module — MCP server for Cursor Cloud Agents
#
# Namespace: services.kurage.mcp.*
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
  mcpCfg = config.services.kurage.mcp;
in {
  options.services.kurage = {
    mcp = mkMcpOptions {
      defaultPackage = pkgs.kurage;
    };
  };

  config = mkIf mcpCfg.enable {
    services.kurage.mcp.serverEntry = mkMcpServerEntry {
      command = "${mcpCfg.package}/bin/kurage";
    };
  };
}
