use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;

use crate::api::types::MessageType;
use crate::auth;
use crate::client::CursorCloudClient;
use crate::config::KurageConfig;

// ── Tool input types ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LaunchInput {
    #[schemars(description = "GitHub repository URL (e.g. https://github.com/org/repo)")]
    repo: String,
    #[schemars(description = "Task description / prompt for the cloud agent")]
    prompt: String,
    #[schemars(description = "Model to use (default: claude-opus-4-6)")]
    model: Option<String>,
    #[schemars(description = "Auto-create a pull request (default: true)")]
    auto_pr: Option<bool>,
    #[schemars(description = "Auto-create a branch (default: true)")]
    auto_branch: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ListInput {
    #[schemars(description = "Maximum number of agents to return (default: 20)")]
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AgentIdInput {
    #[schemars(description = "Agent ID")]
    id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct FollowupInput {
    #[schemars(description = "Agent ID")]
    id: String,
    #[schemars(description = "Followup message to send to the agent")]
    message: String,
}

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

    #[tool(description = "Launch a new Cursor Cloud Agent to work on a GitHub repository. Returns agent ID and initial status.")]
    async fn launch_agent(&self, Parameters(input): Parameters<LaunchInput>) -> String {
        let model_str = input.model.unwrap_or_else(|| self.default_model.clone());
        let req = crate::api::types::LaunchRequest {
            prompt: crate::api::types::PromptSpec {
                text: input.prompt,
                images: None,
            },
            model: if model_str.is_empty() { None } else { Some(model_str) },
            source: crate::api::types::SourceSpec {
                repository: Some(input.repo),
                r#ref: None,
                pr_url: None,
            },
            target: Some(crate::api::types::TargetSpec {
                auto_create_pr: input.auto_pr.unwrap_or(true),
                auto_branch: input.auto_branch.unwrap_or(true),
                open_as_cursor_github_app: false,
                skip_reviewer_request: false,
                branch_name: None,
            }),
            webhook: None,
        };
        match self.client.launch(&req).await {
            Ok(agent) => format_agent(&agent),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List Cursor Cloud Agents. Returns IDs, statuses, repos, and names.")]
    async fn list_agents(&self, Parameters(input): Parameters<ListInput>) -> String {
        let limit = input.limit.unwrap_or(20);
        match self.client.list(limit, None, None).await {
            Ok(list) => {
                if list.agents.is_empty() {
                    return "No agents found.".into();
                }
                let mut out = format!("{} agents:\n", list.agents.len());
                for a in &list.agents {
                    let repo = a
                        .source
                        .as_ref()
                        .map_or("-", |s| s.repository.as_str());
                    let name = truncate(&a.name, 60);
                    out.push_str(&format!(
                        "  {} | {} | {} | {}\n",
                        a.id, a.status, repo, name
                    ));
                }
                out
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get the current status of a Cursor Cloud Agent.")]
    async fn agent_status(&self, Parameters(input): Parameters<AgentIdInput>) -> String {
        match self.client.status(&input.id).await {
            Ok(agent) => format_agent(&agent),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get the conversation log (messages) of a Cursor Cloud Agent.")]
    async fn agent_logs(&self, Parameters(input): Parameters<AgentIdInput>) -> String {
        match self.client.logs(&input.id).await {
            Ok(conv) => {
                if conv.messages.is_empty() {
                    return "No messages yet.".into();
                }
                let mut out = String::new();
                for msg in &conv.messages {
                    let role = match &msg.message_type {
                        Some(MessageType::UserMessage) => "[USER]",
                        Some(MessageType::AssistantMessage) => "[AGENT]",
                        None => "[UNKNOWN]",
                    };
                    out.push_str(&format!("{role}\n{}\n\n", msg.text));
                }
                out
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Stop a running Cursor Cloud Agent.")]
    async fn stop_agent(&self, Parameters(input): Parameters<AgentIdInput>) -> String {
        match self.client.stop(&input.id).await {
            Ok(_) => format!("Stopped agent {}", input.id),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Send a followup message to a Cursor Cloud Agent.")]
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

    #[tool(description = "Delete a Cursor Cloud Agent.")]
    async fn delete_agent(&self, Parameters(input): Parameters<AgentIdInput>) -> String {
        match self.client.delete_agent(&input.id).await {
            Ok(_) => format!("Deleted agent {}", input.id),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List artifacts (files generated) by a Cursor Cloud Agent.")]
    async fn agent_artifacts(&self, Parameters(input): Parameters<AgentIdInput>) -> String {
        match self.client.artifacts(&input.id).await {
            Ok(list) => {
                if list.artifacts.is_empty() {
                    return "No artifacts.".into();
                }
                let mut out = String::new();
                for a in &list.artifacts {
                    out.push_str(&format!(
                        "{} ({} bytes, updated {})\n",
                        a.absolute_path, a.size_bytes, a.updated_at
                    ));
                }
                out
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List available AI models for Cursor Cloud Agents.")]
    async fn list_models(&self, Parameters(_): Parameters<serde_json::Value>) -> String {
        match self.client.models().await {
            Ok(list) => {
                let mut out = String::new();
                for m in &list.models {
                    out.push_str(&format!("{m}\n"));
                }
                out
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List GitHub repositories connected to Cursor.")]
    async fn list_repos(&self, Parameters(_): Parameters<serde_json::Value>) -> String {
        match self.client.repos().await {
            Ok(list) => {
                let mut out = String::new();
                for r in &list.repositories {
                    out.push_str(&format!("{}/{}: {}\n", r.owner, r.name, r.repository));
                }
                out
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get Cursor API key information (current user/account).")]
    async fn whoami(&self, Parameters(_): Parameters<serde_json::Value>) -> String {
        match self.client.me().await {
            Ok(resp) => {
                let mut out = format!("API Key: {}\n", resp.api_key_name);
                out.push_str(&format!("Created: {}\n", resp.created_at));
                if let Some(ref email) = resp.user_email {
                    out.push_str(&format!("Email: {email}\n"));
                }
                out
            }
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

// ── Helpers ─────────────────────────────────────────────────────────────────

fn format_agent(agent: &crate::api::types::Agent) -> String {
    let mut out = String::new();
    out.push_str(&format!("ID: {}\n", agent.id));
    out.push_str(&format!("Name: {}\n", agent.name));
    out.push_str(&format!("Status: {}\n", agent.status));
    if let Some(ref source) = agent.source {
        out.push_str(&format!("Repo: {}\n", source.repository));
        if let Some(ref git_ref) = source.r#ref {
            out.push_str(&format!("Ref: {git_ref}\n"));
        }
    }
    if let Some(ref target) = agent.target {
        out.push_str(&format!("URL: {}\n", target.url));
        if let Some(ref branch) = target.branch_name {
            out.push_str(&format!("Branch: {branch}\n"));
        }
        if let Some(ref pr_url) = target.pr_url {
            out.push_str(&format!("PR: {pr_url}\n"));
        }
    }
    if let Some(ref summary) = agent.summary {
        out.push_str(&format!("Summary: {summary}\n"));
    }
    if let Some(ref created) = agent.created_at {
        out.push_str(&format!("Created: {created}\n"));
    }
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

// ── Entry point ─────────────────────────────────────────────────────────────

pub async fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let server = KurageMcp::new()?.serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}
