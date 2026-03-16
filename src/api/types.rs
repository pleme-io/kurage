use serde::{Deserialize, Serialize};

// ── Launch request ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct LaunchRequest {
    pub prompt: PromptSpec,
    pub model: String,
    pub source: SourceSpec,
    pub target: TargetSpec,
}

#[derive(Debug, Serialize)]
pub struct PromptSpec {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct SourceSpec {
    pub repository: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetSpec {
    pub auto_create_pr: bool,
    pub auto_branch: bool,
}

// ── Followup request ────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FollowupRequest {
    pub prompt: PromptSpec,
}

// ── Agent response (shared by launch, status, list) ─────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub prompt: Option<AgentPrompt>,
    #[serde(default)]
    pub source: Option<AgentSource>,
    #[serde(default)]
    pub target: Option<AgentTarget>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub pull_request: Option<PullRequest>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentPrompt {
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSource {
    #[serde(default)]
    pub repository: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTarget {
    #[serde(default)]
    pub auto_create_pr: bool,
    #[serde(default)]
    pub auto_branch: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequest {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub number: u64,
}

// ── List response ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentList {
    #[serde(default)]
    pub agents: Vec<Agent>,
}

// ── Conversation (logs) ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Conversation {
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Artifacts ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactList {
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artifact {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Models ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelList {
    #[serde(default)]
    pub models: Vec<Model>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Repositories ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoList {
    #[serde(default)]
    pub repositories: Vec<Repo>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub full_name: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Me ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct MeResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

// ── Stop response ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct StopResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

// ── Delete response ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DeleteResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}
