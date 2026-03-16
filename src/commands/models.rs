use crate::client::CursorCloudClient;
use crate::config::OutputFormat;
use crate::error::Result;
use crate::output;

pub async fn run(client: &CursorCloudClient, format: OutputFormat) -> Result<()> {
    let list = client.models().await?;
    output::print_models(&list, format);
    Ok(())
}
