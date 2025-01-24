mod buy_targetted_pubkey;
mod test_quote;

use std::collections::HashMap;

use async_trait::async_trait;
use buy_targetted_pubkey::BuyOnCreationTargettedPubkey;
use clap::ArgMatches;
use test_quote::TestQuote;

#[async_trait]
pub trait Command {
    async fn execute(&self, args: &ArgMatches) -> anyhow::Result<()>;

    fn create(&self) -> clap::Command;

    fn name(&self) -> String;
}

pub fn get_commands() -> HashMap<String, Box<dyn Command>> {
    let mut result = HashMap::new();

    let commands: Vec<Box<dyn Command>> = vec![
        Box::new(BuyOnCreationTargettedPubkey {}),
        Box::new(TestQuote {}),
    ];

    for command in commands {
        result.insert(command.name(), command);
    }

    result
}
