use std::str::FromStr;

use futures::StreamExt;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::{RpcTransactionConfig, RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    signature::Signature,
};
use solana_transaction_status_client_types::{
    EncodedTransaction, UiInstruction, UiMessage, UiParsedInstruction, UiTransactionEncoding,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = "wss://api.mainnet-beta.solana.com";
    let rpc_url = "https://api.mainnet-beta.solana.com";

    raydium(ws_url, rpc_url).await
}

async fn raydium(ws_url: &str, rpc_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ws_client = PubsubClient::new(ws_url).await?;
    let rpc_client = RpcClient::new(rpc_url.to_string());

    let address = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

    let (mut accounts, unsubscriber) = ws_client
        .logs_subscribe(
            RpcTransactionLogsFilter::Mentions(vec![address.to_owned()]),
            RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig {
                    commitment: CommitmentLevel::Confirmed,
                }),
            },
        )
        .await?;

    while let Some(response) = accounts.next().await {
        println!("Slot: {:#?}", response.context.slot);
        let logs = response.value.logs;
        let signature = response.value.signature;
        let signature = Signature::from_str(&signature).unwrap();

        let mut found = false;
        for log in &logs {
            if log.to_lowercase().contains("initialize2") {
                //println!("NEW POOL CREATED : {:#?}", logs);
                found = true;
                break;
            }
        }

        if found {
            let transaction = rpc_client
                .get_transaction_with_config(&signature, RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::JsonParsed),
                    commitment: Some(CommitmentConfig {
                        commitment: CommitmentLevel::Confirmed,
                    }),
                    max_supported_transaction_version: Some(0),
                })
                .await
                .unwrap();

            let transaction = transaction.transaction.transaction;
            if let EncodedTransaction::Json(ui_transaction) = transaction {
                if let UiMessage::Parsed(ui_parsed_message) = ui_transaction.message {
                    for instruction in ui_parsed_message.instructions {
                        if let UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(
                            parsed_instruction,
                        )) = instruction
                        {
                            if parsed_instruction.program_id == address {
                                println!("New Pool Detected");
                                println!("  Tx Signature: {:#?}", &signature);
                                println!("  Token A: {:#?}", parsed_instruction.accounts[8]);
                                println!("  Token B: {:#?}", parsed_instruction.accounts[9]);
                            }
                        }
                    }
                }
            }

            break;
        }
    }

    unsubscriber().await;

    Ok(())
}
