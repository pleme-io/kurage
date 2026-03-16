use crate::client::CursorCloudClient;
use crate::config::OutputFormat;
use crate::error::Result;
use crate::output;

pub async fn run(client: &CursorCloudClient, format: OutputFormat) -> Result<()> {
    let resp = client.me().await?;
    output::print_me(&resp, format);
    Ok(())
}
