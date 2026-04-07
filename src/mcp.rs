use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;

use crate::auth;
use crate::client::CursorCloudClient;
use crate::config::KurageConfig;
use crate::format;

// ── MCP tool input types ────────────────────────────────────────────────────
//
// These derive from the OpenAPI spec schemas. Each field maps to a spec
// property with its description preserved for schemars → MCP tool schema.

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LaunchInput {
    #[schemars(description = "GitHub repository URL (e.g. https://github.com/org/repo). Required unless pr_url is provided.")]
    repo: Option<String>,
    #[schemars(description = "Task description / prompt for the cloud agent")]
    prompt: String,
    #[schemars(description = "Model ID to use (e.g. claude-4-sonnet). Omit to use configured default.")]
    model: Option<String>,
    #[schemars(description = "Git ref (branch/tag) to use as the base branch")]
    git_ref: Option<String>,
    #[schemars(description = "GitHub pull request URL. When provided, agent works on this PR's repo and branches. Mutually exclusive with repo/git_ref.")]
    pr_url: Option<String>,
    #[schemars(description = "Auto-create a pull request when agent completes (default: true)")]
    auto_pr: Option<bool>,
    #[schemars(description = "Auto-create a new branch (default: true)")]
    auto_branch: Option<bool>,
    #[schemars(description = "Custom branch name for the agent to create")]
    branch_name: Option<String>,
    #[schemars(description = "Open PR as the Cursor GitHub App instead of as the user (default: false)")]
    open_as_cursor_github_app: Option<bool>,
    #[schemars(description = "Skip adding the user as a reviewer to the PR (default: false)")]
    skip_reviewer_request: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ListInput {
    #[schemars(description = "Maximum number of agents to return (default: 20, max: 100)")]
    limit: Option<u32>,
    #[schemars(description = "Pagination cursor from previous response's nextCursor field")]
    cursor: Option<String>,
    #[schemars(description = "Filter agents by pull request URL")]
    pr_url: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AgentIdInput {
    #[schemars(description = "Unique agent identifier (e.g. bc_abc123)")]
    id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct FollowupInput {
    #[schemars(description = "Unique agent identifier (e.g. bc_abc123)")]
    id: String,
    #[schemars(description = "Followup instruction for the agent")]
    message: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DownloadArtifactInput {
    #[schemars(description = "Unique agent identifier (e.g. bc_abc123)")]
    id: String,
    #[schemars(description = "Absolute artifact path from the list artifacts response")]
    path: String,
}

/// Empty input for tools that take no parameters.
///
/// MCP requires every tool to have an `inputSchema` that is a valid JSON Schema
/// object with `type: "object"`. Using `serde_json::Value` here generates a
/// schema of `{"title":"AnyValue"}` with no `type` field, which Claude Code's
/// MCP loader rejects, causing the entire kurage server to surface zero tools.
#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
struct NoInput {}

// ── MCP Server ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct KurageMcp {
    client: CursorCloudClient,
    default_model: String,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl KurageMcp {
    fn new() -> Result<Self, String> {
        let config = KurageConfig::load();
        let api_key = auth::resolve_api_key(None, &config).map_err(|e| e.to_string())?;
        let client =
            CursorCloudClient::new(&config.api_url, &api_key).map_err(|e| e.to_string())?;

        Ok(Self {
            client,
            default_model: config.default_model.clone(),
            tool_router: Self::tool_router(),
        })
    }

    #[tool(description = "Launch a new Cursor Cloud Agent. Provide repo URL or PR URL, plus a task prompt. Returns agent ID and status.")]
    async fn launch_agent(&self, Parameters(input): Parameters<LaunchInput>) -> String {
        let model_val = input.model.filter(|s| !s.is_empty());
        let req = crate::api::types::LaunchRequest {
            prompt: crate::api::types::PromptSpec {
                text: input.prompt,
                images: None,
            },
            model: model_val.or_else(|| Some(self.default_model.clone())),
            source: crate::api::types::SourceSpec {
                repository: input.repo,
                r#ref: input.git_ref,
                pr_url: input.pr_url,
            },
            target: Some(crate::api::types::TargetSpec {
                auto_create_pr: input.auto_pr.unwrap_or(true),
                auto_branch: input.auto_branch.unwrap_or(true),
                open_as_cursor_github_app: input.open_as_cursor_github_app.unwrap_or(false),
                skip_reviewer_request: input.skip_reviewer_request.unwrap_or(false),
                branch_name: input.branch_name,
            }),
            webhook: None,
        };
        match self.client.launch(&req).await {
            Ok(agent) => format::format_agent(&agent),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List Cursor Cloud Agents with optional pagination and PR filter.")]
    async fn list_agents(&self, Parameters(input): Parameters<ListInput>) -> String {
        let limit = input.limit.unwrap_or(20);
        match self.client.list(limit, input.cursor.as_deref(), input.pr_url.as_deref()).await {
            Ok(list) => format::format_agent_list(&list),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get the current status and details of a Cursor Cloud Agent.")]
    async fn agent_status(&self, Parameters(input): Parameters<AgentIdInput>) -> String {
        match self.client.status(&input.id).await {
            Ok(agent) => format::format_agent(&agent),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get the conversation log (messages) of a Cursor Cloud Agent.")]
    async fn agent_logs(&self, Parameters(input): Parameters<AgentIdInput>) -> String {
        match self.client.logs(&input.id).await {
            Ok(conv) => format::format_conversation(&conv),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Stop a running Cursor Cloud Agent. Stopped agents cannot be resumed.")]
    async fn stop_agent(&self, Parameters(input): Parameters<AgentIdInput>) -> String {
        match self.client.stop(&input.id).await {
            Ok(_) => format!("Stopped agent {}", input.id),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Send a followup instruction to a Cursor Cloud Agent.")]
    async fn agent_followup(&self, Parameters(input): Parameters<FollowupInput>) -> String {
        let req = crate::api::types::FollowupRequest {
            prompt: crate::api::types::PromptSpec {
                text: input.message,
                images: None,
            },
        };
        match self.client.followup(&input.id, &req).await {
            Ok(resp) => format!("Followup sent to agent {}", resp.id),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Delete a Cursor Cloud Agent permanently.")]
    async fn delete_agent(&self, Parameters(input): Parameters<AgentIdInput>) -> String {
        match self.client.delete_agent(&input.id).await {
            Ok(_) => format!("Deleted agent {}", input.id),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List artifacts (files) generated by a Cursor Cloud Agent. Returns at most 100 artifacts.")]
    async fn agent_artifacts(&self, Parameters(input): Parameters<AgentIdInput>) -> String {
        match self.client.artifacts(&input.id).await {
            Ok(list) => format::format_artifacts(&list),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a temporary download URL for an artifact from a Cursor Cloud Agent. URL expires in 15 minutes.")]
    async fn download_artifact(&self, Parameters(input): Parameters<DownloadArtifactInput>) -> String {
        match self.client.download_artifact(&input.id, &input.path).await {
            Ok(resp) => format!("Download URL: {}\nExpires: {}", resp.url, resp.expires_at),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List available AI models for Cursor Cloud Agents.")]
    async fn list_models(&self, Parameters(_): Parameters<NoInput>) -> String {
        match self.client.models().await {
            Ok(list) => format::format_models(&list),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List GitHub repositories connected to Cursor. Rate limited: 1/min, 30/hour.")]
    async fn list_repos(&self, Parameters(_): Parameters<NoInput>) -> String {
        match self.client.repos().await {
            Ok(list) => format::format_repos(&list),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get Cursor API key information (key name, creation date, user email).")]
    async fn whoami(&self, Parameters(_): Parameters<NoInput>) -> String {
        match self.client.me().await {
            Ok(resp) => format::format_me(&resp),
            Err(e) => format!("Error: {e}"),
        }
    }
}

#[tool_handler]
impl ServerHandler for KurageMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Cursor Cloud Agents — launch, monitor, and manage AI coding agents on GitHub repositories via Cursor's cloud infrastructure."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

// ── Entry point ─────────────────────────────────────────────────────────────

pub async fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let server = KurageMcp::new()?.serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}
