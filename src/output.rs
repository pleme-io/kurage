use crate::api::types::{Agent, AgentList, ArtifactList, Conversation, MessageType, ModelList, RepoList};
use crate::config::OutputFormat;
use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};

pub fn print_json<T: serde::Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
}

pub fn print_agent(agent: &Agent, format: OutputFormat) {
    match format {
        OutputFormat::Json => print_json(agent),
        OutputFormat::Pretty | OutputFormat::Table => {
            println!("ID:      {}", agent.id);
            println!("Name:    {}", agent.name);
            println!("Status:  {}", agent.status);
            if let Some(ref source) = agent.source {
                println!("Repo:    {}", source.repository);
                if let Some(ref git_ref) = source.r#ref {
                    println!("Ref:     {git_ref}");
                }
            }
            if let Some(ref target) = agent.target {
                println!("URL:     {}", target.url);
                if let Some(ref branch) = target.branch_name {
                    println!("Branch:  {branch}");
                }
                if let Some(ref pr_url) = target.pr_url {
                    println!("PR:      {pr_url}");
                }
            }
            if let Some(ref summary) = agent.summary {
                println!("Summary: {summary}");
            }
            if let Some(ref created) = agent.created_at {
                println!("Created: {created}");
            }
        }
    }
}

pub fn print_agent_list(list: &AgentList, format: OutputFormat) {
    match format {
        OutputFormat::Json => print_json(list),
        OutputFormat::Pretty => {
            if list.agents.is_empty() {
                println!("No agents found.");
                return;
            }
            for agent in &list.agents {
                let repo = agent
                    .source
                    .as_ref()
                    .map_or("-", |s| s.repository.as_str());
                let name = if agent.name.len() > 50 {
                    format!("{}...", &agent.name[..47])
                } else {
                    agent.name.clone()
                };
                println!("{} | {:10} | {} | {}", agent.id, agent.status, repo, name);
            }
        }
        OutputFormat::Table => {
            if list.agents.is_empty() {
                println!("No agents found.");
                return;
            }
            let mut table = Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);
            table.set_header(["ID", "Status", "Name", "Repo"]);
            for agent in &list.agents {
                let repo = agent
                    .source
                    .as_ref()
                    .map_or("-".to_string(), |s| s.repository.clone());
                let name = if agent.name.len() > 40 {
                    format!("{}...", &agent.name[..37])
                } else {
                    agent.name.clone()
                };
                let status = agent.status.to_string();
                table.add_row([
                    &agent.id,
                    &status,
                    &name,
                    &repo,
                ]);
            }
            println!("{table}");
        }
    }
}

pub fn print_conversation(conv: &Conversation, format: OutputFormat) {
    match format {
        OutputFormat::Json => print_json(conv),
        OutputFormat::Pretty | OutputFormat::Table => {
            if conv.messages.is_empty() {
                println!("No messages yet.");
                return;
            }
            for msg in &conv.messages {
                let role_label = match &msg.message_type {
                    Some(MessageType::UserMessage) => "[USER]",
                    Some(MessageType::AssistantMessage) => "[AGENT]",
                    None => "[UNKNOWN]",
                };
                println!("{role_label}");
                println!("{}", msg.text);
                println!();
            }
        }
    }
}

pub fn print_models(list: &ModelList, format: OutputFormat) {
    match format {
        OutputFormat::Json => print_json(list),
        OutputFormat::Pretty | OutputFormat::Table => {
            for model in &list.models {
                println!("{model}");
            }
        }
    }
}

pub fn print_repos(list: &RepoList, format: OutputFormat) {
    match format {
        OutputFormat::Json => print_json(list),
        OutputFormat::Pretty | OutputFormat::Table => {
            for repo in &list.repositories {
                println!("{}/{}: {}", repo.owner, repo.name, repo.repository);
            }
        }
    }
}

pub fn print_artifacts(list: &ArtifactList, format: OutputFormat) {
    match format {
        OutputFormat::Json => print_json(list),
        OutputFormat::Pretty | OutputFormat::Table => {
            if list.artifacts.is_empty() {
                println!("No artifacts.");
                return;
            }
            for artifact in &list.artifacts {
                println!("{} ({} bytes, updated {})", artifact.absolute_path, artifact.size_bytes, artifact.updated_at);
            }
        }
    }
}
