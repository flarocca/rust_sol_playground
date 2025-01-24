use anyhow::Context;
use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches};

use crate::raydium::event_processors::EventProcessor;

use super::Command;

pub struct TestQuote;

#[async_trait]
impl Command for TestQuote {
    async fn execute(&self, args: &ArgMatches) -> anyhow::Result<()> {
        let rpc_url = args
            .get_one::<String>("rpc-url")
            .with_context(|| "RPC URL is required")?;
        let ws_url = args
            .get_one::<String>("ws-url")
            .with_context(|| "WS URL is required")?;
        let signature = args
            .get_one::<String>("signature")
            .with_context(|| "Signature is not valid")?;

        let raydium_processor = EventProcessor::new(rpc_url, ws_url).await?;

        let pool = raydium_processor
            .get_pool_from_create_transaction(signature)
            .await?;

        let quote = raydium_processor.get_market_keys(&pool).await?;

        println!("RAYDIUM - Quote: {:?}", quote);

        Ok(())
    }

    fn create(&self) -> clap::Command {
        clap::Command::new("test-quote")
            .about("Test the quote of a pool")
            .long_flag("test-quote")
            .arg(
                Arg::new("signature")
                    .long("signature")
                    .short('s')
                    .required(true)
                    .action(ArgAction::Set)
                    .help("The signature of the transaction where the pool was created"),
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
    }

    fn name(&self) -> String {
        "test-quote".to_string()
    }
}
