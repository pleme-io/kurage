use crate::api::types::{Agent, AgentList, ArtifactList, Conversation, ModelList, RepoList};
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
            println!("Status:  {}", agent.status);
            println!("Model:   {}", agent.model);
            if let Some(ref prompt) = agent.prompt {
                let text = if prompt.text.len() > 80 {
                    format!("{}...", &prompt.text[..77])
                } else {
                    prompt.text.clone()
                };
                println!("Prompt:  {text}");
            }
            if let Some(ref source) = agent.source {
                println!("Repo:    {}", source.repository);
            }
            if let Some(ref created) = agent.created_at {
                println!("Created: {created}");
            }
            if let Some(ref pr) = agent.pull_request {
                println!("PR:      {} (#{}) ", pr.url, pr.number);
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
                let prompt = agent
                    .prompt
                    .as_ref()
                    .map_or("-".to_string(), |p| {
                        if p.text.len() > 50 {
                            format!("{}...", &p.text[..47])
                        } else {
                            p.text.clone()
                        }
                    });
                println!("{} | {:10} | {} | {}", agent.id, agent.status, repo, prompt);
            }
        }
        OutputFormat::Table => {
            if list.agents.is_empty() {
                println!("No agents found.");
                return;
            }
            let mut table = Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);
            table.set_header(["ID", "Status", "Model", "Repo", "Prompt"]);
            for agent in &list.agents {
                let repo = agent
                    .source
                    .as_ref()
                    .map_or("-".to_string(), |s| s.repository.clone());
                let prompt = agent
                    .prompt
                    .as_ref()
                    .map_or("-".to_string(), |p| {
                        if p.text.len() > 40 {
                            format!("{}...", &p.text[..37])
                        } else {
                            p.text.clone()
                        }
                    });
                table.add_row([
                    &agent.id,
                    &agent.status,
                    &agent.model,
                    &repo,
                    &prompt,
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
                let role_label = match msg.role.as_str() {
                    "user" => "[USER]",
                    "assistant" => "[AGENT]",
                    "system" => "[SYSTEM]",
                    _ => &msg.role,
                };
                println!("{role_label}");
                println!("{}", msg.content);
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
                println!("{}: {}", model.id, model.name);
            }
        }
    }
}

pub fn print_repos(list: &RepoList, format: OutputFormat) {
    match format {
        OutputFormat::Json => print_json(list),
        OutputFormat::Pretty | OutputFormat::Table => {
            for repo in &list.repositories {
                let name = if repo.full_name.is_empty() {
                    &repo.name
                } else {
                    &repo.full_name
                };
                println!("{name}: {}", repo.url);
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
                println!("{} ({})", artifact.path, artifact.r#type);
            }
        }
    }
}
