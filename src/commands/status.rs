use clap::Args as ClapArgs;

use crate::client::CursorCloudClient;
use crate::config::OutputFormat;
use crate::error::Result;
use crate::output;

#[derive(ClapArgs)]
pub struct Args {
    /// Agent ID
    pub id: String,

    /// Poll for updates at the configured interval
    #[arg(long, short)]
    pub follow: bool,
}

pub async fn run(
    args: Args,
    client: &CursorCloudClient,
    format: OutputFormat,
    poll_interval: u64,
) -> Result<()> {
    if args.follow {
        loop {
            let agent = client.status(&args.id).await?;
            // Clear screen for follow mode
            print!("\x1b[2J\x1b[H");
            output::print_agent(&agent, format);

            let terminal = matches!(
                agent.status,
                crate::api::types::AgentStatus::Finished
                    | crate::api::types::AgentStatus::Error
                    | crate::api::types::AgentStatus::Expired
            );
            if terminal {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(poll_interval)).await;
        }
    } else {
        let agent = client.status(&args.id).await?;
        output::print_agent(&agent, format);
    }
    Ok(())
}
