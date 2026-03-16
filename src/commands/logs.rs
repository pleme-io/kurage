use clap::Args as ClapArgs;

use crate::api::types::MessageType;
use crate::client::CursorCloudClient;
use crate::config::OutputFormat;
use crate::error::Result;
use crate::output;

#[derive(ClapArgs)]
pub struct Args {
    /// Agent ID
    pub id: String,

    /// Poll for new messages at the configured interval
    #[arg(long, short)]
    pub follow: bool,
}

pub async fn run(
    args: Args,
    client: &CursorCloudClient,
    format: OutputFormat,
    poll_interval: u64,
) -> Result<()> {
    let mut seen = 0;

    loop {
        let conv = client.logs(&args.id).await?;

        if args.follow {
            // Only print new messages
            for msg in conv.messages.iter().skip(seen) {
                let role_label = match &msg.message_type {
                    Some(MessageType::UserMessage) => "[USER]",
                    Some(MessageType::AssistantMessage) => "[AGENT]",
                    None => "[UNKNOWN]",
                };
                if format == OutputFormat::Json {
                    output::print_json(msg);
                } else {
                    println!("{role_label}");
                    println!("{}", msg.text);
                    println!();
                }
            }
            seen = conv.messages.len();

            // Check if agent is done
            let agent = client.status(&args.id).await?;
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
        } else {
            output::print_conversation(&conv, format);
            break;
        }
    }
    Ok(())
}
