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
            images: None,
        },
    };
    let resp = client.followup(&args.id, &req).await?;
    if format == OutputFormat::Json {
        output::print_json(&resp);
    } else {
        println!("Followup sent to agent {}", resp.id);
    }
    Ok(())
}
