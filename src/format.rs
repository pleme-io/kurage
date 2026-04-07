use crate::api::types::{Agent, AgentList, ArtifactList, Conversation, MeResponse, MessageType, ModelList, RepoList};
use std::fmt::Write;

/// Truncate a string to `max` characters, appending `...` if shortened.
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

/// Render a single agent as human-readable multi-line text.
pub fn format_agent(agent: &Agent) -> String {
    let mut out = String::with_capacity(512);
    let _ = writeln!(out, "ID:      {}", agent.id);
    let _ = writeln!(out, "Name:    {}", agent.name);
    let _ = writeln!(out, "Status:  {}", agent.status);
    if let Some(ref source) = agent.source {
        let _ = writeln!(out, "Repo:    {}", source.repository);
        if let Some(ref git_ref) = source.r#ref {
            let _ = writeln!(out, "Ref:     {git_ref}");
        }
    }
    if let Some(ref target) = agent.target {
        if !target.url.is_empty() {
            let _ = writeln!(out, "URL:     {}", target.url);
        }
        if let Some(ref branch) = target.branch_name {
            let _ = writeln!(out, "Branch:  {branch}");
        }
        if let Some(ref pr_url) = target.pr_url {
            let _ = writeln!(out, "PR:      {pr_url}");
        }
    }
    if let Some(ref summary) = agent.summary {
        let _ = writeln!(out, "Summary: {summary}");
    }
    if let Some(ref created) = agent.created_at {
        let _ = writeln!(out, "Created: {created}");
    }
    out
}

/// Render an agent list as a summary table with optional pagination cursor.
pub fn format_agent_list(list: &AgentList) -> String {
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
        let _ = writeln!(out, "  {} | {:10} | {} | {}", a.id, a.status, repo, name);
    }
    if let Some(ref cursor) = list.next_cursor {
        let _ = writeln!(out, "\n[next page: cursor={cursor}]");
    }
    out
}

/// Render a conversation as role-prefixed message blocks.
pub fn format_conversation(conv: &Conversation) -> String {
    if conv.messages.is_empty() {
        return "No messages yet.".into();
    }
    let mut out = String::with_capacity(4096);
    for msg in &conv.messages {
        let role = match &msg.message_type {
            Some(MessageType::UserMessage) => "[USER]",
            Some(MessageType::AssistantMessage) => "[AGENT]",
            None => "[UNKNOWN]",
        };
        let _ = writeln!(out, "{role}");
        let _ = writeln!(out, "{}", msg.text);
        out.push('\n');
    }
    out
}

/// Render a model list as one model ID per line.
pub fn format_models(list: &ModelList) -> String {
    let mut out = String::new();
    for m in &list.models {
        let _ = writeln!(out, "{m}");
    }
    out
}

/// Render a repository list as `owner/name: url` lines.
pub fn format_repos(list: &RepoList) -> String {
    let mut out = String::new();
    for r in &list.repositories {
        let _ = writeln!(out, "{}/{}: {}", r.owner, r.name, r.repository);
    }
    out
}

/// Render an artifact list as `path (size, updated)` lines.
pub fn format_artifacts(list: &ArtifactList) -> String {
    if list.artifacts.is_empty() {
        return "No artifacts.".into();
    }
    let mut out = String::new();
    for a in &list.artifacts {
        let _ = writeln!(out, "{} ({} bytes, updated {})", a.absolute_path, a.size_bytes, a.updated_at);
    }
    out
}

/// Render API key / user info from the `/v0/me` endpoint.
pub fn format_me(resp: &MeResponse) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "API Key: {}", resp.api_key_name);
    let _ = writeln!(out, "Created: {}", resp.created_at);
    if let Some(ref email) = resp.user_email {
        let _ = writeln!(out, "Email:   {email}");
    }
    out
}
