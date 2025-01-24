use anyhow::Context;
use serde::{Deserialize, Serialize};
use solana_account_decoder::parse_token::UiTokenAmount;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature,
    transaction::TransactionVersion,
};
use solana_transaction_status_client_types::{
    EncodedTransaction, UiTransactionEncoding, UiTransactionStatusMeta,
};
use std::{str::FromStr, time::Duration};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub slot: u64,
    pub block_time: Option<i64>,
    pub transaction: EncodedTransaction,
    pub metadata: Option<UiTransactionStatusMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<TransactionVersion>,
    pub signature: Signature,
}

pub struct SolanaApi {
    rpc_client: RpcClient,
}

impl SolanaApi {
    pub fn new(
        rpc_url: &str,
        _timeout: Option<Duration>,
        commitment_config: Option<CommitmentConfig>,
    ) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            commitment_config.unwrap_or_default(),
        );

        Self { rpc_client }
    }

    pub async fn get_transaction(&self, signature: &str) -> anyhow::Result<Transaction> {
        let signature = Signature::from_str(signature).unwrap();

        let transaction = self
            .rpc_client
            .get_transaction_with_config(
                &signature,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::JsonParsed),
                    commitment: None,
                    max_supported_transaction_version: Some(0),
                },
            )
            .await
            .with_context(|| format!("Failed to get transaction: {}", signature))?;

        Ok(Transaction {
            slot: transaction.slot,
            block_time: transaction.block_time,
            transaction: transaction.transaction.transaction,
            metadata: transaction.transaction.meta,
            version: transaction.transaction.version,
            signature,
        })
    }

    pub async fn get_token_balance(&self, token_account: &str) -> anyhow::Result<UiTokenAmount> {
        let token_account_balance = self
            .rpc_client
            .get_token_account_balance(&Pubkey::from_str(token_account).unwrap())
            .await
            .with_context(|| "Failed to get token account balance")?;

        Ok(token_account_balance)
    }

    pub async fn get_account<T>(&self, account: &Pubkey) -> anyhow::Result<T>
    where
        T: Clone,
    {
        let account = self
            .rpc_client
            .get_account(account)
            .await
            .with_context(|| format!("Error getting account {:?}", account))?;

        let account_data = account.data.as_slice();
        let ret = unsafe { &*(&account_data[0] as *const u8 as *const T) };

        Ok(ret.clone())
    }

    pub async fn get_account_data(&self, account: &Pubkey) -> anyhow::Result<Vec<u8>> {
        let data = self
            .rpc_client
            .get_account_data(account)
            .await
            .with_context(|| format!("Error getting account data for account {:?}", account))?;

        Ok(data)
    }
}
