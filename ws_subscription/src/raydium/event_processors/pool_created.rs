use anyhow::{anyhow, Context};
use futures::StreamExt;
use safe_transmute::{transmute_many_pedantic, transmute_one_pedantic, transmute_to_bytes};
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
};
use solana_transaction_status_client_types::{
    EncodedTransaction, UiInstruction, UiMessage, UiParsedInstruction,
};
use std::{borrow::Cow, str::FromStr};

use crate::{
    api::solana_rpc::Transaction,
    raydium::{
        event_processors::RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID,
        models::{AccountFlag, AmmKeys, Market, MarketKeys, MarketState, MarketStateV2, Pool},
    },
};

pub const AUTHORITY_AMM: &[u8] = b"amm authority";
pub const ACCOUNT_HEAD_PADDING: &[u8; 5] = b"serum";
pub const ACCOUNT_TAIL_PADDING: &[u8; 7] = b"padding";

use super::{EventProcessor, WSOL};

impl EventProcessor {
    pub async fn buy_new_pool(
        &self,
        target: Pubkey,
        amount: u64,
        signature: &str,
        simulate_only: bool,
    ) -> anyhow::Result<()> {
        println!("RAYDIUM - Signature: {:#?}", &signature);

        let transaction = self.solana_api.get_transaction(signature).await?;

        let Transaction {
            transaction,
            metadata,
            signature,
            ..
        } = transaction;

        println!("\nMetadata: {:#?}\n", &metadata);
        println!("\nTransaction: {:#?}\n", &transaction);

        if let EncodedTransaction::Json(ui_transaction) = transaction {
            if let UiMessage::Parsed(ui_parsed_message) = ui_transaction.message {
                for instruction in ui_parsed_message.instructions {
                    if let UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(
                        parsed_instruction,
                    )) = instruction
                    {
                        if parsed_instruction.program_id
                            == RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID.to_string()
                        {
                            println!("------------ New Pool Detected ------------");
                            println!("    Tx Signature: {:#?}", &signature);
                            println!("    Instruction: {:#?}", &parsed_instruction);

                            let amm = &parsed_instruction.accounts[4];
                            let amm_authority = &parsed_instruction.accounts[5];
                            let amm_open_orders = &parsed_instruction.accounts[6];
                            let amm_lp_mint = &parsed_instruction.accounts[7];
                            let amm_coin_mint = &parsed_instruction.accounts[8];
                            let amm_coin_vault = &parsed_instruction.accounts[10];
                            let amm_pc_mint = &parsed_instruction.accounts[9];
                            let amm_pc_vault = &parsed_instruction.accounts[11];
                            let amm_target = &parsed_instruction.accounts[13];
                            let market_program = &parsed_instruction.accounts[15];
                            let market = &parsed_instruction.accounts[16];

                            if *amm_coin_mint != target.to_string()
                                || *amm_pc_mint != target.to_string()
                            {
                                return anyhow::Result::Err(anyhow::anyhow!(
                                    "Target not found in pool creation"
                                ));
                            }

                            let amm_keys = AmmKeys {
                                amm_pool: Pubkey::from_str(amm).unwrap(),
                                amm_coin_mint: Pubkey::from_str(amm_coin_mint).unwrap(),
                                amm_pc_mint: Pubkey::from_str(amm_pc_mint).unwrap(),
                                amm_authority: Pubkey::from_str(amm_authority).unwrap(),
                                amm_target: Pubkey::from_str(amm_target).unwrap(),
                                amm_coin_vault: Pubkey::from_str(amm_coin_vault).unwrap(),
                                amm_pc_vault: Pubkey::from_str(amm_pc_vault).unwrap(),
                                amm_lp_mint: Pubkey::from_str(amm_lp_mint).unwrap(),
                                amm_open_order: Pubkey::from_str(amm_open_orders).unwrap(),
                                market_program: Pubkey::from_str(market_program).unwrap(),
                                market: Pubkey::from_str(market).unwrap(),
                                nonce: 0,
                            };

                            let amm_coin_initial_balance =
                                self.solana_api.get_token_balance(amm_coin_vault).await?;

                            println!(
                                "    AMM Coin Initial Balance: {:#?}",
                                amm_coin_initial_balance
                            );

                            let amm_pc_initial_balance =
                                self.solana_api.get_token_balance(amm_pc_vault).await?;
                            println!("    AMM PC Initial Balance: {:#?}", amm_pc_initial_balance);

                            let pool = Pool {
                                amm: amm_keys,
                                initial_coin_balance: amm_coin_initial_balance
                                    .amount
                                    .parse()
                                    .with_context(|| "Failed to parse initial coin balance")?,
                                initial_pc_balance: amm_pc_initial_balance
                                    .amount
                                    .parse()
                                    .with_context(|| "Failed to parse initial pc balance")?,
                            };

                            println!("    Pool: {:#?}", &pool);
                            println!("-------------------------------------------");

                            //let mut pools = self.pools.lock().await;
                            //pools.insert(pool.amm.amm_pool, pool.clone());

                            //self.subscribe_to_new_pool(pool.amm.amm_pool).await?;
                            self.buy(target, pool, amount, simulate_only).await?;

                            return Ok(());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn buy(
        &self,
        owner: Keypair,
        target: Pubkey,
        pool: Pool,
        amount: u64,
        simulate_only: bool,
    ) -> anyhow::Result<()> {
        let token_mint_input = WSOL;
        let token_mint_output = target;
        //    let owner = Keypair::read_from_file(keypair_file_path).expect("Error parsing private key");
        //
        //    let token_account_input =
        //        Pubkey::from_str("GKNaPCWkQfg8KK8gxtnCrVWsgLPUj9AhoywTQ7yX9GBE").unwrap();
        //    let token_account_output =
        //        Pubkey::from_str("4oSZPX4QJkpy12VaGuJjRiMaFxAMFdKZpJznSbEFm61h").unwrap();
        //
        //    let (amm_keys, market_keys) = get_serum_quote_via_raydium_api(
        //        &token_mint_output.to_string(),
        //        &token_mint_input.to_string(),
        //    )
        //    .await
        //    .unwrap();
        //
        //    //println!("AMM Keys: {:#?}", &amm_keys);
        //    //println!("Market Keys: {:#?}", &market_keys);
        //
        let (amm_keys, market_keys) = self
            .get_serum_quote(rpc_client, &amm_keys.amm_pool)
            .await
            .unwrap();
        //
        //    //println!("AMM Keys: {:#?}", &amm_keys);
        //    //println!("Market Keys: {:#?}", &market_keys);
        //
        //    let amount_in: u64 = 143594511; //1_000_000;
        //    let min_amount_out: u64 = 0; //900_000;
        //
        //    let instruction_tag = 9u8; // "Swap" tag, https://github.com/reactive-biscuit/raydium-amm/blob/ae039d21cd49ef670d76b3a1cf5485ae0213dc5e/program/src/instruction.rs#L487
        //    let mut swap_data = vec![instruction_tag];
        //    swap_data.extend_from_slice(&amount_in.to_le_bytes());
        //    swap_data.extend_from_slice(&min_amount_out.to_le_bytes());
        //
        //    let swap_accounts = vec![
        //        AccountMeta::new(TOKEN_PROGRAM, false),
        //        AccountMeta::new(amm_keys.amm_pool, false),
        //        AccountMeta::new(amm_keys.amm_authority, false),
        //        AccountMeta::new(amm_keys.amm_open_order, false),
        //        AccountMeta::new(amm_keys.amm_coin_vault, false),
        //        AccountMeta::new(amm_keys.amm_pc_vault, false),
        //        AccountMeta::new(amm_keys.market_program, false),
        //        AccountMeta::new(amm_keys.market, false),
        //        AccountMeta::new(market_keys.bids, false),
        //        AccountMeta::new(market_keys.asks, false),
        //        AccountMeta::new(market_keys.event_queue, false),
        //        AccountMeta::new(market_keys.coin_vault, false),
        //        AccountMeta::new(market_keys.pc_vault, false),
        //        AccountMeta::new(market_keys.vault_signer_key, false),
        //        AccountMeta::new(token_account_output, false),
        //        AccountMeta::new(token_account_input, false),
        //        AccountMeta::new(owner.pubkey(), true),
        //    ];
        //    let compute_units_price = ComputeBudgetInstruction::set_compute_unit_price(3_000_000);
        //
        //    let swap_ix = Instruction {
        //        program_id: RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID,
        //        accounts: swap_accounts,
        //        data: swap_data,
        //    };
        //
        //    let recent_blockhash = rpc_client.get_latest_blockhash().await.unwrap();
        //    let mut message = VersionedMessage::Legacy(Message::new(&[compute_units_price, swap_ix], None));
        //    message.set_recent_blockhash(recent_blockhash);
        //
        //    let signers: [&dyn Signer; 1] = [&owner];
        //
        //    let transaction = VersionedTransaction::try_new(message.clone(), &signers).unwrap();
        //    let simulation_result = rpc_client
        //        .simulate_transaction(&transaction)
        //        .await
        //        .expect("Simulation failed");
        //
        //    println!("Simulation result: {:#?}", simulation_result);
        //
        //    let recent_blockhash = rpc_client.get_latest_blockhash().await.unwrap();
        //    message.set_recent_blockhash(recent_blockhash);
        //    let transaction = VersionedTransaction::try_new(message.clone(), &signers).unwrap();
        //
        //    let result = rpc_client.send_and_confirm_transaction(&transaction).await;
        //    println!("Transaction result {:#?}", result);
        //
        //    Ok(())
        Ok(())
    }

    async fn get_serum_quote(&self, pool: &Pool) -> anyhow::Result<(AmmKeys, MarketKeys)> {
        //let amm_info = self
        //    .solana_api
        //    .get_account::<AmmInfo>(&pool.amm.amm_pool)
        //    .await
        //    .unwrap()
        //    .unwrap();
        //
        //let amm_keys = AmmKeys {
        //    amm_pool: *amm_pool,
        //    amm_target: amm_info.target_orders,
        //    amm_coin_vault: amm_info.coin_vault,
        //    amm_pc_vault: amm_info.pc_vault,
        //    amm_lp_mint: amm_info.lp_mint,
        //    amm_open_order: amm_info.open_orders,
        //    amm_coin_mint: amm_info.coin_vault_mint,
        //    amm_pc_mint: amm_info.pc_vault_mint,
        //    amm_authority: compute_amm_authority_id(
        //        &RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID,
        //        amm_info.nonce as u8,
        //    )?,
        //    market: amm_info.market,
        //    market_program: amm_info.market_program,
        //    nonce: amm_info.nonce as u8,
        //};

        let account_data = self.solana_api.get_account_data(&pool.amm.market).await?;

        Ok((amm_keys, market_keys))
    }

    pub async fn process_new_pool(&self, signature: &str) -> anyhow::Result<()> {
        println!("Signature: {:#?}", &signature);

        let transaction = self.solana_api.get_transaction(signature).await?;

        let Transaction {
            transaction,
            metadata,
            signature,
            ..
        } = transaction;

        println!("\nMetadata: {:#?}\n", &metadata);
        println!("\nTransaction: {:#?}\n", &transaction);

        if let EncodedTransaction::Json(ui_transaction) = transaction {
            if let UiMessage::Parsed(ui_parsed_message) = ui_transaction.message {
                for instruction in ui_parsed_message.instructions {
                    if let UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(
                        parsed_instruction,
                    )) = instruction
                    {
                        if parsed_instruction.program_id
                            == RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID.to_string()
                        {
                            println!("------------ New Pool Detected ------------");
                            println!("    Tx Signature: {:#?}", &signature);
                            println!("    Instruction: {:#?}", &parsed_instruction);

                            let amm = &parsed_instruction.accounts[4];
                            let amm_authority = &parsed_instruction.accounts[5];
                            let amm_open_orders = &parsed_instruction.accounts[6];
                            let amm_lp_mint = &parsed_instruction.accounts[7];
                            let amm_coin_mint = &parsed_instruction.accounts[8];
                            let amm_coin_vault = &parsed_instruction.accounts[10];
                            let amm_pc_mint = &parsed_instruction.accounts[9];
                            let amm_pc_vault = &parsed_instruction.accounts[11];
                            let amm_target = &parsed_instruction.accounts[13];
                            let market_program = &parsed_instruction.accounts[15];
                            let market = &parsed_instruction.accounts[16];

                            let amm_keys = AmmKeys {
                                amm_pool: Pubkey::from_str(amm).unwrap(),
                                amm_coin_mint: Pubkey::from_str(amm_coin_mint).unwrap(),
                                amm_pc_mint: Pubkey::from_str(amm_pc_mint).unwrap(),
                                amm_authority: Pubkey::from_str(amm_authority).unwrap(),
                                amm_target: Pubkey::from_str(amm_target).unwrap(),
                                amm_coin_vault: Pubkey::from_str(amm_coin_vault).unwrap(),
                                amm_pc_vault: Pubkey::from_str(amm_pc_vault).unwrap(),
                                amm_lp_mint: Pubkey::from_str(amm_lp_mint).unwrap(),
                                amm_open_order: Pubkey::from_str(amm_open_orders).unwrap(),
                                market_program: Pubkey::from_str(market_program).unwrap(),
                                market: Pubkey::from_str(market).unwrap(),
                                nonce: 0,
                            };

                            let amm_coin_initial_balance =
                                self.solana_api.get_token_balance(amm_coin_vault).await?;

                            println!(
                                "    AMM Coin Initial Balance: {:#?}",
                                amm_coin_initial_balance
                            );

                            let amm_pc_initial_balance =
                                self.solana_api.get_token_balance(amm_pc_vault).await?;
                            println!("    AMM PC Initial Balance: {:#?}", amm_pc_initial_balance);

                            let pool = Pool {
                                amm: amm_keys,
                                initial_coin_balance: amm_coin_initial_balance
                                    .amount
                                    .parse()
                                    .with_context(|| "Failed to parse initial coin balance")?,
                                initial_pc_balance: amm_pc_initial_balance
                                    .amount
                                    .parse()
                                    .with_context(|| "Failed to parse initial pc balance")?,
                            };

                            println!("    Pool: {:#?}", &pool);
                            println!("-------------------------------------------");

                            let mut pools = self.pools.lock().await;
                            pools.insert(pool.amm.amm_pool, pool.clone());

                            self.subscribe_to_new_pool(pool.amm.amm_pool).await?;

                            return Ok(());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn subscribe_to_new_pool(&self, pubkey: Pubkey) -> anyhow::Result<()> {
        println!("Subscribing to new pool: {:#?}", &pubkey);
        let (mut accounts, unsubscriber) = self
            .ws_client
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![pubkey.to_string()]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig {
                        commitment: CommitmentLevel::Processed,
                    }),
                },
            )
            .await?;

        let mut subscriptions = self.subscriptions.lock().await;
        subscriptions.insert(pubkey, unsubscriber);

        while let Some(response) = accounts.next().await {
            let logs = response.value.logs.clone();
            let signature = response.value.signature.clone();

            for log in &logs {
                if log.to_lowercase().ends_with("swap") {
                    println!("Signature: {:#?}", &signature);
                    println!("------------------ Swap Detected ------------------");
                    println!("Log: {:#?}", &log);
                    //found = true;
                    //break;
                }

                if log.to_lowercase().ends_with("swap2")
                    || log.to_lowercase().ends_with("multiswap")
                {
                    println!("Signature: {:#?}", &signature);
                    println!("Log: {:#?}", &log);
                }
            }
        }

        Ok(())
    }
    pub fn remove_dex_account_padding<'a>(data: &'a [u8]) -> anyhow::Result<Cow<'a, [u64]>> {
        let head = &data[..ACCOUNT_HEAD_PADDING.len()];
        if data.len() < ACCOUNT_HEAD_PADDING.len() + ACCOUNT_TAIL_PADDING.len() {
            return Err(anyhow!(
                "dex account length {} is too small to contain valid padding",
                data.len()
            ));
        }

        if head != ACCOUNT_HEAD_PADDING {
            return Err(anyhow!("dex account head padding mismatch".to_string()));
        }

        let tail = &data[data.len() - ACCOUNT_TAIL_PADDING.len()..];
        if tail != ACCOUNT_TAIL_PADDING {
            return Err(anyhow!("dex account tail padding mismatch".to_string()));
        }

        let inner_data_range =
            ACCOUNT_HEAD_PADDING.len()..(data.len() - ACCOUNT_TAIL_PADDING.len());

        let inner: &'a [u8] = &data[inner_data_range];
        let words: Cow<'a, [u64]> = match transmute_many_pedantic::<u64>(inner) {
            Ok(word_slice) => Cow::Borrowed(word_slice),
            Err(transmute_error) => {
                let word_vec = transmute_error
                    .copy()
                    .with_context(|| "Error reading account data")?;
                Cow::Owned(word_vec)
            }
        };

        Ok(words)
    }

    fn get_market_keys(account_data: [u8]) -> anyhow::Result<MarketKeys> {
        let words = Self::remove_dex_account_padding(&account_data)?;

        let market_state: MarketState = {
            let account_flags = Market::account_flags(&account_data)?;
            if account_flags.intersects(AccountFlag::Permissioned) {
                let state = transmute_one_pedantic::<MarketStateV2>(transmute_to_bytes(&words))
                    .map_err(|e| e.without_src())?;
                //state.check_flags(true)?;
                state.inner
            } else {
                let state = transmute_one_pedantic::<MarketState>(transmute_to_bytes(&words))
                    .map_err(|e| e.without_src())?;
                //state.check_flags(true)?;
                state
            }
        };
        let vault_signer_key = gen_vault_signer_key(
            market_state.vault_signer_nonce,
            &amm_keys.market,
            &amm_keys.market_program,
        )?;

        let market_keys = MarketKeys {
            event_queue: Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.event_q)))
                .unwrap(),
            bids: Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.bids))).unwrap(),
            asks: Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.asks))).unwrap(),
            coin_vault: Pubkey::try_from(transmute_one_to_bytes(&identity(
                market_state.coin_vault,
            )))
            .unwrap(),
            pc_vault: Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.pc_vault)))
                .unwrap(),
            vault_signer_key,
        };
        //let account_data = self.solana_api.get_account_data(&pool.amm.market).await?;
        //
        //let account_flags = Market::account_flags(&account_data)?;
        //
        //if !account_flags.contains(AccountFlag::Initialized) {
        //    return Err("Market account not initialized".to_string());
        //}
        //
        //let market_state = MarketStateV2::from_bytes(&account_data)?;
        //
        //market_state.check_flags(false)?;
        //
        //let market_keys = MarketKeys {
        //    bids: market_state.inner.bids,
        //    asks: market_state.inner.asks,
        //    event_queue: market_state.inner.event_queue,
        //    coin_vault: market_state.inner.coin_vault,
        //    pc_vault: market_state.inner.pc_vault,
        //    vault_signer_key: gen_vault_signer_key(market_state.inner.vault_signer_nonce, &pool.amm.market, &RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID)?,
        //};

        Ok(market_keys)
    }
}
