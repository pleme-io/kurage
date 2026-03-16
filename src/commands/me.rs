use crate::client::CursorCloudClient;
use crate::config::OutputFormat;
use crate::error::Result;
use crate::output;

pub async fn run(client: &CursorCloudClient, format: OutputFormat) -> Result<()> {
    let resp = client.me().await?;
    if format == OutputFormat::Json {
        output::print_json(&resp.data);
    } else {
        println!("{}", serde_json::to_string_pretty(&resp.data).unwrap_or_default());
    }
    Ok(())
}
