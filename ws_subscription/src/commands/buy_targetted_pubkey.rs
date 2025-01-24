use std::str::FromStr;

use anyhow::Context;
use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches};
use solana_sdk::pubkey::Pubkey;

use crate::raydium::{self, event_processors::EventProcessor};

use super::Command;

pub struct BuyOnCreationTargettedPubkey;

#[async_trait]
impl Command for BuyOnCreationTargettedPubkey {
    async fn execute(&self, args: &ArgMatches) -> anyhow::Result<()> {
        let rpc_url = args
            .get_one::<String>("rpc-url")
            .with_context(|| "RPC URL is required")?;
        let ws_url = args
            .get_one::<String>("ws-url")
            .with_context(|| "WS URL is required")?;
        let target_pubkey = args
            .get_one::<String>("target-pubkey")
            .with_context(|| "WS URL is required")?
            .parse::<Pubkey>()
            .with_context(|| "Failed to parse target pubkey")?;
        let amount = args
            .get_one::<String>("amount")
            .expect("WS URL is required")
            .parse::<u64>()
            .with_context(|| "Failed to parse amount")?;

        let raydium_processor = EventProcessor::new(rpc_url, ws_url).await?;
        raydium_processor
            .execute_on_creation(target_pubkey, amount, true)
            .await?;

        Ok(())
    }

    fn create(&self) -> clap::Command {
        clap::Command::new("buy-on-creation-targetted-pubkey")
            .about("Buy a target token or pool as soon as it is created")
            .long_flag("buy-on-creation-targetted-pubkey")
            .arg(
                Arg::new("target-pubkey")
                    .long("target-pubkey")
                    .short('t')
                    .required(true)
                    .action(ArgAction::Set)
                    .help("The pubkey of the target token or pool"),
            )
            .arg(
                Arg::new("ws-url")
                    .long("ws-url")
                    .required(true)
                    .action(ArgAction::Set)
                    .help("The URL of the Solana WebSocket endpoint"),
            )
            .arg(
                Arg::new("rpc-url")
                    .long("rpc-url")
                    .required(true)
                    .action(ArgAction::Set)
                    .help("The URL of the Solana RPC endpoint"),
            )
            .arg(
                Arg::new("amount")
                    .long("amount")
                    .short('a')
                    .required(true)
                    .action(ArgAction::Set)
                    .help("The amount of the target token to buy"),
            )
            .arg(
                Arg::new("simulate-only")
                    .long("simulate-only")
                    .action(ArgAction::SetFalse)
                    .help("Simulate the buy without actually executing it"),
            )
    }

    fn name(&self) -> String {
        "buy-on-creation-targetted-pubkey".to_string()
    }
}
