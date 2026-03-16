use clap::Args as ClapArgs;

use crate::client::CursorCloudClient;
use crate::config::OutputFormat;
use crate::error::Result;
use crate::output;

#[derive(ClapArgs)]
pub struct Args {
    /// Maximum number of agents to return
    #[arg(long, short, default_value = "20")]
    pub limit: u32,
}

pub async fn run(args: Args, client: &CursorCloudClient, format: OutputFormat) -> Result<()> {
    let list = client.list(args.limit).await?;
    output::print_agent_list(&list, format);
    Ok(())
}
