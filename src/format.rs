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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::*;

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_exact_length() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_long_string() {
        let result = truncate("hello world!", 8);
        assert_eq!(result, "hello...");
        assert!(result.len() <= 8);
    }

    #[test]
    fn truncate_very_short_max() {
        let result = truncate("hello", 3);
        assert_eq!(result, "...");
    }

    fn make_agent(id: &str, status: AgentStatus) -> Agent {
        Agent {
            id: id.into(),
            name: "test-agent".into(),
            status,
            source: None,
            target: None,
            summary: None,
            created_at: None,
            extra: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    #[test]
    fn format_agent_minimal() {
        let agent = make_agent("bc_1", AgentStatus::Running);
        let out = format_agent(&agent);
        assert!(out.contains("ID:      bc_1"));
        assert!(out.contains("Name:    test-agent"));
        assert!(out.contains("Status:  RUNNING"));
        assert!(!out.contains("Repo:"));
        assert!(!out.contains("Summary:"));
    }

    #[test]
    fn format_agent_with_source_and_target() {
        let agent = Agent {
            id: "bc_2".into(),
            name: "full-agent".into(),
            status: AgentStatus::Finished,
            source: Some(AgentSource {
                repository: "https://github.com/org/repo".into(),
                r#ref: Some("main".into()),
                extra: serde_json::Value::Object(serde_json::Map::new()),
            }),
            target: Some(AgentTarget {
                branch_name: Some("cursor/fix".into()),
                url: "https://cursor.com/agent/bc_2".into(),
                pr_url: Some("https://github.com/org/repo/pull/1".into()),
                auto_create_pr: true,
                open_as_cursor_github_app: false,
                skip_reviewer_request: false,
                extra: serde_json::Value::Object(serde_json::Map::new()),
            }),
            summary: Some("Fixed it".into()),
            created_at: Some("2025-01-01".into()),
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        let out = format_agent(&agent);
        assert!(out.contains("Repo:    https://github.com/org/repo"));
        assert!(out.contains("Ref:     main"));
        assert!(out.contains("URL:     https://cursor.com/agent/bc_2"));
        assert!(out.contains("Branch:  cursor/fix"));
        assert!(out.contains("PR:      https://github.com/org/repo/pull/1"));
        assert!(out.contains("Summary: Fixed it"));
        assert!(out.contains("Created: 2025-01-01"));
    }

    #[test]
    fn format_agent_list_empty() {
        let list = AgentList {
            agents: vec![],
            next_cursor: None,
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        assert_eq!(format_agent_list(&list), "No agents found.");
    }

    #[test]
    fn format_agent_list_with_agents() {
        let list = AgentList {
            agents: vec![make_agent("bc_1", AgentStatus::Running)],
            next_cursor: None,
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        let out = format_agent_list(&list);
        assert!(out.starts_with("1 agents:"));
        assert!(out.contains("bc_1"));
        assert!(out.contains("RUNNING"));
    }

    #[test]
    fn format_agent_list_with_pagination() {
        let list = AgentList {
            agents: vec![make_agent("bc_1", AgentStatus::Finished)],
            next_cursor: Some("next_abc".into()),
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        let out = format_agent_list(&list);
        assert!(out.contains("[next page: cursor=next_abc]"));
    }

    #[test]
    fn format_conversation_empty() {
        let conv = Conversation {
            id: "bc_1".into(),
            messages: vec![],
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        assert_eq!(format_conversation(&conv), "No messages yet.");
    }

    #[test]
    fn format_conversation_with_messages() {
        let conv = Conversation {
            id: "bc_1".into(),
            messages: vec![
                Message {
                    id: "m1".into(),
                    message_type: Some(MessageType::UserMessage),
                    text: "Fix the bug".into(),
                    extra: serde_json::Value::Object(serde_json::Map::new()),
                },
                Message {
                    id: "m2".into(),
                    message_type: Some(MessageType::AssistantMessage),
                    text: "Done!".into(),
                    extra: serde_json::Value::Object(serde_json::Map::new()),
                },
                Message {
                    id: "m3".into(),
                    message_type: None,
                    text: "unknown".into(),
                    extra: serde_json::Value::Object(serde_json::Map::new()),
                },
            ],
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        let out = format_conversation(&conv);
        assert!(out.contains("[USER]"));
        assert!(out.contains("Fix the bug"));
        assert!(out.contains("[AGENT]"));
        assert!(out.contains("Done!"));
        assert!(out.contains("[UNKNOWN]"));
    }

    #[test]
    fn format_models_list() {
        let list = ModelList {
            models: vec!["claude-4-sonnet".into(), "gpt-4o".into()],
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        let out = format_models(&list);
        assert!(out.contains("claude-4-sonnet"));
        assert!(out.contains("gpt-4o"));
    }

    #[test]
    fn format_repos_list() {
        let list = RepoList {
            repositories: vec![Repo {
                owner: "pleme-io".into(),
                name: "kurage".into(),
                repository: "https://github.com/pleme-io/kurage".into(),
                extra: serde_json::Value::Object(serde_json::Map::new()),
            }],
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        let out = format_repos(&list);
        assert!(out.contains("pleme-io/kurage: https://github.com/pleme-io/kurage"));
    }

    #[test]
    fn format_artifacts_empty() {
        let list = ArtifactList {
            artifacts: vec![],
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        assert_eq!(format_artifacts(&list), "No artifacts.");
    }

    #[test]
    fn format_artifacts_with_items() {
        let list = ArtifactList {
            artifacts: vec![Artifact {
                absolute_path: "/opt/file.txt".into(),
                size_bytes: 1024,
                updated_at: "2025-01-01".into(),
                extra: serde_json::Value::Object(serde_json::Map::new()),
            }],
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        let out = format_artifacts(&list);
        assert!(out.contains("/opt/file.txt"));
        assert!(out.contains("1024 bytes"));
        assert!(out.contains("updated 2025-01-01"));
    }

    #[test]
    fn format_me_with_email() {
        let resp = MeResponse {
            api_key_name: "my-key".into(),
            created_at: "2025-01-01".into(),
            user_email: Some("user@example.com".into()),
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        let out = format_me(&resp);
        assert!(out.contains("API Key: my-key"));
        assert!(out.contains("Created: 2025-01-01"));
        assert!(out.contains("Email:   user@example.com"));
    }

    #[test]
    fn format_me_without_email() {
        let resp = MeResponse {
            api_key_name: "k".into(),
            created_at: "2025-01-01".into(),
            user_email: None,
            extra: serde_json::Value::Object(serde_json::Map::new()),
        };
        let out = format_me(&resp);
        assert!(out.contains("API Key: k"));
        assert!(!out.contains("Email:"));
    }
}
