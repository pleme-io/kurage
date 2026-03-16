use clap::Args as ClapArgs;

use crate::api::types::{FollowupRequest, PromptSpec};
use crate::client::CursorCloudClient;
use crate::config::OutputFormat;
use crate::error::Result;
use crate::output;

#[derive(ClapArgs)]
pub struct Args {
    /// Agent ID
    pub id: String,

    /// Followup message
    #[arg(trailing_var_arg = true)]
    pub message: Vec<String>,
}

pub async fn run(args: Args, client: &CursorCloudClient, format: OutputFormat) -> Result<()> {
    let req = FollowupRequest {
        prompt: PromptSpec {
            text: args.message.join(" "),
        },
    };
    let agent = client.followup(&args.id, &req).await?;
    output::print_agent(&agent, format);
    Ok(())
}
