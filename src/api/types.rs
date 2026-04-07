use serde::{Deserialize, Serialize};

// ── Agent status enum ──────────────────────────────────────────────────────────

/// Agent lifecycle status.
/// Maps to the `status` enum in the `OpenAPI` spec: RUNNING, FINISHED, ERROR, CREATING, EXPIRED.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentStatus {
    Running,
    Finished,
    Error,
    Creating,
    Expired,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "RUNNING"),
            Self::Finished => write!(f, "FINISHED"),
            Self::Error => write!(f, "ERROR"),
            Self::Creating => write!(f, "CREATING"),
            Self::Expired => write!(f, "EXPIRED"),
        }
    }
}

// ── Message type enum ──────────────────────────────────────────────────────────

/// Conversation message type.
/// Maps to the `type` enum: `user_message`, `assistant_message`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    UserMessage,
    AssistantMessage,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserMessage => write!(f, "user_message"),
            Self::AssistantMessage => write!(f, "assistant_message"),
        }
    }
}

// ── Image types (for prompt attachments) ───────────────────────────────────────

/// Image dimension (width x height).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDimension {
    pub width: u32,
    pub height: u32,
}

/// Base64-encoded image with optional dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// Base64 encoded image data.
    pub data: String,
    /// Optional image dimensions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimension: Option<ImageDimension>,
}

// ── Create agent request (POST /v0/agents) ─────────────────────────────────────

/// Prompt specification for create and followup requests.
#[derive(Debug, Serialize, Deserialize)]
pub struct PromptSpec {
    /// The task or instructions for the agent to execute.
    pub text: String,
    /// Optional array of base64-encoded images (max 5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<Image>>,
}

/// Source repository specification.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSpec {
    /// The GitHub repository URL. Required unless prUrl is provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Git ref (branch/tag) to use as the base branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    /// GitHub pull request URL. Mutually exclusive with repository/ref.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pr_url: Option<String>,
}

/// Target configuration for the agent's output.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct TargetSpec {
    /// Whether to automatically create a pull request when the agent completes.
    #[serde(default)]
    pub auto_create_pr: bool,
    /// Whether to open the PR as the Cursor GitHub App instead of as the user.
    #[serde(default)]
    pub open_as_cursor_github_app: bool,
    /// Whether to skip adding the user as a reviewer to the pull request.
    #[serde(default)]
    pub skip_reviewer_request: bool,
    /// Custom branch name for the agent to create.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    /// Whether to create a new branch (true) or use the PR's existing head branch (false).
    #[serde(default)]
    pub auto_branch: bool,
}

/// Webhook configuration for agent status change notifications.
#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookSpec {
    /// URL to receive webhook notifications about agent status changes.
    pub url: String,
    /// Secret key for webhook payload verification (min 32 chars).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

/// Request body for POST /v0/agents (launch a new agent).
#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchRequest {
    /// The prompt with task instructions and optional images.
    pub prompt: PromptSpec,
    /// Model ID to use, or "default". When omitted, Cursor resolves defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Source repository and ref configuration.
    pub source: SourceSpec,
    /// Target branch and PR configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<TargetSpec>,
    /// Webhook configuration for status change notifications.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook: Option<WebhookSpec>,
}

// ── Followup request (POST /v0/agents/{id}/followup) ───────────────────────────

/// Request body for POST /v0/agents/{id}/followup.
#[derive(Debug, Serialize, Deserialize)]
pub struct FollowupRequest {
    /// The followup instruction with optional images.
    pub prompt: PromptSpec,
}

// ── Agent response types (shared by create, get, list) ─────────────────────────

/// Source information in agent responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSource {
    /// The GitHub repository URL.
    #[serde(default)]
    pub repository: String,
    /// Git ref (branch/tag) used as the base branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Target information in agent responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTarget {
    /// The Git branch name where the agent is working.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    /// URL to view the agent in Cursor Web.
    #[serde(default)]
    pub url: String,
    /// URL of the pull request, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pr_url: Option<String>,
    /// Whether a pull request will be automatically created.
    #[serde(default)]
    pub auto_create_pr: bool,
    /// Whether the PR will be opened as the Cursor GitHub App.
    #[serde(default)]
    pub open_as_cursor_github_app: bool,
    /// Whether to skip adding the user as a reviewer.
    #[serde(default)]
    pub skip_reviewer_request: bool,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Full agent object returned by create, get, and list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    /// Unique identifier for the cloud agent (e.g. `bc_abc123`).
    pub id: String,
    /// Name for the agent.
    #[serde(default)]
    pub name: String,
    /// Current lifecycle status.
    #[serde(default)]
    pub status: AgentStatus,
    /// Source repository information.
    #[serde(default)]
    pub source: Option<AgentSource>,
    /// Target branch and PR information.
    #[serde(default)]
    pub target: Option<AgentTarget>,
    /// Summary of the agent's work (present when FINISHED).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// When the agent was created (ISO 8601).
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self::Creating
    }
}

// ── List agents response (GET /v0/agents) ──────────────────────────────────────

/// Response from GET /v0/agents.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentList {
    /// List of agents.
    #[serde(default)]
    pub agents: Vec<Agent>,
    /// Cursor for fetching the next page of results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Conversation (GET /v0/agents/{id}/conversation) ────────────────────────────

/// A single conversation message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for the message.
    #[serde(default)]
    pub id: String,
    /// Type of message (`user_message` or `assistant_message`).
    #[serde(default, rename = "type")]
    #[allow(clippy::struct_field_names)]
    pub message_type: Option<MessageType>,
    /// The content of the message.
    #[serde(default)]
    pub text: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Conversation response from GET /v0/agents/{id}/conversation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Conversation {
    /// Agent ID this conversation belongs to.
    #[serde(default)]
    pub id: String,
    /// Array of conversation messages ordered chronologically.
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Artifacts (GET /v0/agents/{id}/artifacts) ──────────────────────────────────

/// A single artifact generated by the cloud agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    /// Absolute artifact path in the cloud agent environment.
    #[serde(default)]
    pub absolute_path: String,
    /// Artifact file size in bytes.
    #[serde(default)]
    pub size_bytes: u64,
    /// Last modified timestamp for the artifact (ISO 8601).
    #[serde(default)]
    pub updated_at: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Response from GET /v0/agents/{id}/artifacts.
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactList {
    /// Artifacts generated by the cloud agent (at most 100).
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Download artifact (GET /v0/agents/{id}/artifacts/download) ─────────────────

/// Response from GET /v0/agents/{id}/artifacts/download.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetArtifactResponse {
    /// Temporary 15-minute presigned S3 URL for downloading the artifact.
    pub url: String,
    /// When the presigned download URL expires (ISO 8601).
    pub expires_at: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Models (GET /v0/models) ────────────────────────────────────────────────────

/// Response from GET /v0/models.
/// Note: models is an array of string model IDs, not objects.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelList {
    /// Array of recommended explicit model IDs.
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Repositories (GET /v0/repositories) ────────────────────────────────────────

/// A single GitHub repository accessible to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    /// The owner of the repository (user or organization).
    #[serde(default)]
    pub owner: String,
    /// The name of the repository.
    #[serde(default)]
    pub name: String,
    /// The full URL to the GitHub repository.
    #[serde(default)]
    pub repository: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Response from GET /v0/repositories.
#[derive(Debug, Serialize, Deserialize)]
pub struct RepoList {
    /// Array of GitHub repositories the user has access to.
    #[serde(default)]
    pub repositories: Vec<Repo>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Me (GET /v0/me) ────────────────────────────────────────────────────────────

/// Response from GET /v0/me.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    /// The name of the API key.
    #[serde(default)]
    pub api_key_name: String,
    /// When the API key was created (ISO 8601).
    #[serde(default)]
    pub created_at: String,
    /// Email address of the user who owns the API key (if available).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_email: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Simple action responses (stop, delete, followup) ───────────────────────────

/// Response from POST /v0/agents/{id}/stop.
#[derive(Debug, Serialize, Deserialize)]
pub struct StopResponse {
    /// Agent ID that was stopped.
    pub id: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Response from DELETE /v0/agents/{id}.
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResponse {
    /// Agent ID that was deleted.
    pub id: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Response from POST /v0/agents/{id}/followup.
#[derive(Debug, Serialize, Deserialize)]
pub struct FollowupResponse {
    /// Agent ID that received the followup.
    pub id: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Error response ─────────────────────────────────────────────────────────────

/// Error detail nested inside the API error envelope.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Human-readable error message.
    #[serde(default)]
    pub message: String,
    /// Machine-readable error code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Top-level API error response: `{ "error": { "message": "...", "code": "..." } }`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: ErrorDetail,
}
