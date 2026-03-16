use crate::client::CursorCloudClient;
use crate::config::OutputFormat;
use crate::error::Result;
use crate::output;

pub async fn run(client: &CursorCloudClient, format: OutputFormat) -> Result<()> {
    let list = client.repos().await?;
    output::print_repos(&list, format);
    Ok(())
}
