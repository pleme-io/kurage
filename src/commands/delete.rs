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
    let resp = client.delete_agent(&args.id).await?;
    if format == OutputFormat::Json {
        output::print_json(&resp.data);
    } else {
        println!("Deleted agent {}", args.id);
    }
    Ok(())
}
