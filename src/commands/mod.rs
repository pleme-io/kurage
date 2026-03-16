use clap::{Parser, Subcommand};

pub mod artifacts;
pub mod delete;
pub mod followup;
pub mod launch;
pub mod list;
pub mod logs;
pub mod me;
pub mod models;
pub mod repos;
pub mod status;
pub mod stop;

#[derive(Parser)]
#[command(
    name = "kurage",
    version,
    about = "Cursor Cloud Agents CLI + MCP server"
)]
pub struct Cli {
    /// Cursor API key (overrides env and file)
    #[arg(long, global = true, env = "CURSOR_API_KEY")]
    pub api_key: Option<String>,

    /// API base URL
    #[arg(long, global = true)]
    pub api_url: Option<String>,

    /// Output format: json, pretty, table
    #[arg(long, global = true)]
    pub output: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Launch a new cloud agent
    Launch(launch::Args),
    /// List cloud agents
    List(list::Args),
    /// Get agent status
    Status(status::Args),
    /// Get agent conversation logs
    Logs(logs::Args),
    /// Stop a running agent
    Stop(stop::Args),
    /// Send a followup message to an agent
    Followup(followup::Args),
    /// Delete an agent
    Delete(delete::Args),
    /// List agent artifacts
    Artifacts(artifacts::Args),
    /// List available models
    Models,
    /// List connected repositories
    Repos,
    /// Show API key info
    Me,
}
