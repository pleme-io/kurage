use clap::Parser;
use std::process::ExitCode;

mod api;
mod auth;
mod client;
mod commands;
mod config;
mod error;
mod format;
mod mcp;
mod output;

use commands::{Cli, Command};
use config::{KurageConfig, OutputFormat};

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // No subcommand → MCP server mode (stdio)
    let Some(command) = cli.command else {
        init_tracing(true);
        if let Err(e) = mcp::run().await {
            eprintln!("MCP server error: {e}");
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    };

    init_tracing(false);

    let config = KurageConfig::load();

    // Resolve output format: CLI flag > config > default
    let format = cli
        .output
        .as_deref()
        .and_then(parse_output_format)
        .unwrap_or(config.output);

    // Resolve API key and build client
    let api_key = match auth::resolve_api_key(cli.api_key.as_deref(), &config) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let api_url = cli.api_url.as_deref().unwrap_or(&config.api_url);
    let client = match client::CursorCloudClient::new(api_url, &api_key) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let result = match command {
        Command::Launch(args) => {
            commands::launch::run(args, &client, &config.default_model, format).await
        }
        Command::List(args) => commands::list::run(args, &client, format).await,
        Command::Status(args) => {
            commands::status::run(args, &client, format, config.poll_interval).await
        }
        Command::Logs(args) => {
            commands::logs::run(args, &client, format, config.poll_interval).await
        }
        Command::Stop(args) => commands::stop::run(args, &client, format).await,
        Command::Followup(args) => commands::followup::run(args, &client, format).await,
        Command::Delete(args) => commands::delete::run(args, &client, format).await,
        Command::Artifacts(args) => commands::artifacts::run(args, &client, format).await,
        Command::Models => commands::models::run(&client, format).await,
        Command::Repos => commands::repos::run(&client, format).await,
        Command::Me => commands::me::run(&client, format).await,
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn parse_output_format(s: &str) -> Option<OutputFormat> {
    match s {
        "json" => Some(OutputFormat::Json),
        "pretty" => Some(OutputFormat::Pretty),
        "table" => Some(OutputFormat::Table),
        _ => None,
    }
}

fn init_tracing(json: bool) {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    if json {
        fmt().json().with_env_filter(filter).with_writer(std::io::stderr).init();
    } else {
        fmt().with_env_filter(filter).init();
    }
}
