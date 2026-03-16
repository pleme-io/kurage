use clap::Args as ClapArgs;

use crate::api::types::{LaunchRequest, PromptSpec, SourceSpec, TargetSpec};
use crate::client::CursorCloudClient;
use crate::config::OutputFormat;
use crate::error::Result;
use crate::output;

#[derive(ClapArgs)]
pub struct Args {
    /// GitHub repository URL (e.g. https://github.com/pleme-io/nexus)
    #[arg(long, short)]
    pub repo: String,

    /// Prompt / task description for the agent
    #[arg(trailing_var_arg = true)]
    pub prompt: Vec<String>,

    /// Model to use (default from config)
    #[arg(long, short)]
    pub model: Option<String>,

    /// Auto-create a pull request
    #[arg(long, default_value = "true")]
    pub auto_pr: bool,

    /// Auto-create a branch
    #[arg(long, default_value = "true")]
    pub auto_branch: bool,
}

pub async fn run(args: Args, client: &CursorCloudClient, model: &str, format: OutputFormat) -> Result<()> {
    let prompt_text = args.prompt.join(" ");
    let model = args.model.as_deref().unwrap_or(model);

    let req = LaunchRequest {
        prompt: PromptSpec {
            text: prompt_text,
            images: None,
        },
        model: if model.is_empty() { None } else { Some(model.to_string()) },
        source: SourceSpec {
            repository: Some(args.repo),
            r#ref: None,
            pr_url: None,
        },
        target: Some(TargetSpec {
            auto_create_pr: args.auto_pr,
            auto_branch: args.auto_branch,
            open_as_cursor_github_app: false,
            skip_reviewer_request: false,
            branch_name: None,
        }),
        webhook: None,
    };

    let agent = client.launch(&req).await?;
    output::print_agent(&agent, format);
    Ok(())
}
