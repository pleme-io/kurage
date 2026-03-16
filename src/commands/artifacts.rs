use clap::Args as ClapArgs;

use crate::client::CursorCloudClient;
use crate::config::OutputFormat;
use crate::error::Result;
use crate::output;

#[derive(ClapArgs)]
pub struct Args {
    /// Agent ID
    pub id: String,
}

pub async fn run(args: Args, client: &CursorCloudClient, format: OutputFormat) -> Result<()> {
    let list = client.artifacts(&args.id).await?;
    output::print_artifacts(&list, format);
    Ok(())
}
