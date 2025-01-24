use std::{collections::HashMap, str::FromStr};

use anyhow::Context;
use futures::{future::BoxFuture, StreamExt};
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::Signature,
};
use tokio::sync::Mutex;

use crate::api::solana_rpc::SolanaApi;

use super::models::Pool;

pub mod new_swap;
pub mod pool_created;

const RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
const WSOL = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");
//const TOKEN_PROGRAM: Pubkey = solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
//const SERUM_PROGRAM: Pubkey = solana_sdk::pubkey!("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX");

type Unsubscriber = Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send>;

pub struct EventProcessor {
    ws_client: PubsubClient,
    solana_api: SolanaApi,
    pools: Mutex<HashMap<Pubkey, Pool>>,
    subscriptions: Mutex<HashMap<Pubkey, Unsubscriber>>,
}

impl EventProcessor {
    pub async fn new(rpc_url: &str, ws_url: &str) -> anyhow::Result<Self> {
        let ws_client    = PubsubClient::new(ws_url)
            .await
            .with_context(|| "Failed to create WS client")?;
        let solana_api = SolanaApi::new(rpc_url, None, Some(CommitmentConfig {
            commitment: CommitmentLevel::Processed,
        }));

        let pools = Mutex::new(HashMap::new());
        let subscriptions = Mutex::new(HashMap::new());

        Ok(Self {
            solana_api,
            ws_client,
            pools,
            subscriptions,
        })
    }

    pub async fn execute_on_creation(
        &self,
        target: Pubkey,
        amount: u64,
        simulate_only: bool,
    ) -> anyhow::Result<()> {
        println!("RAYDIUM - Starting event processor for target: {}", target);

        let (mut accounts, unsubscriber) = self
            .ws_client
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![
                    //RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID.to_string()
                    target.to_string(),
                ]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig {
                        commitment: CommitmentLevel::Processed,
                    }),
                },
            )
            .await?;

        while let Some(response) = accounts.next().await {
            let logs = response.value.logs;
            let signature = response.value.signature;

            for log in &logs {
                if log.to_lowercase().contains("initialize2") {
                    println!("RAYDIUM - Pool creation detected for key {:#?}", target);
                    self.buy_new_pool(target, amount, &signature, simulate_only)
                        .await?;
                }
            }
        }

        unsubscriber().await;

        Ok(())
    }
}
