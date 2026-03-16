use crate::api::types::{Agent, AgentList, ArtifactList, Conversation, MeResponse, ModelList, RepoList};
use crate::config::OutputFormat;
use crate::format;
use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};

pub fn print_json<T: serde::Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
}

pub fn print_agent(agent: &Agent, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => print_json(agent),
        OutputFormat::Pretty | OutputFormat::Table => print!("{}", format::format_agent(agent)),
    }
}

pub fn print_agent_list(list: &AgentList, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => print_json(list),
        OutputFormat::Pretty => print!("{}", format::format_agent_list(list)),
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
                let name = format::truncate(&agent.name, 40);
                let status = agent.status.to_string();
                table.add_row([&agent.id, &status, &name, &repo]);
            }
            println!("{table}");
        }
    }
}

pub fn print_conversation(conv: &Conversation, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => print_json(conv),
        OutputFormat::Pretty | OutputFormat::Table => print!("{}", format::format_conversation(conv)),
    }
}

pub fn print_models(list: &ModelList, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => print_json(list),
        OutputFormat::Pretty | OutputFormat::Table => print!("{}", format::format_models(list)),
    }
}

pub fn print_repos(list: &RepoList, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => print_json(list),
        OutputFormat::Pretty | OutputFormat::Table => print!("{}", format::format_repos(list)),
    }
}

pub fn print_artifacts(list: &ArtifactList, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => print_json(list),
        OutputFormat::Pretty | OutputFormat::Table => print!("{}", format::format_artifacts(list)),
    }
}

pub fn print_me(resp: &MeResponse, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => print_json(resp),
        OutputFormat::Pretty | OutputFormat::Table => print!("{}", format::format_me(resp)),
    }
}
