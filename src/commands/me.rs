use crate::client::CursorCloudClient;
use crate::config::OutputFormat;
use crate::error::Result;
use crate::output;

pub async fn run(client: &CursorCloudClient, format: OutputFormat) -> Result<()> {
    let resp = client.me().await?;
    if format == OutputFormat::Json {
        output::print_json(&resp);
    } else {
        println!("API Key: {}", resp.api_key_name);
        println!("Created: {}", resp.created_at);
        if let Some(ref email) = resp.user_email {
            println!("Email:   {email}");
        }
    }
    Ok(())
}
