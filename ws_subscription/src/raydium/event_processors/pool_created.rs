use anyhow::{anyhow, Context};
use bytemuck::bytes_of;
use futures::StreamExt;
use safe_transmute::{
    transmute_many_pedantic, transmute_one_pedantic, transmute_one_to_bytes, transmute_to_bytes,
};
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    program_error::ProgramError,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
};
use solana_transaction_status_client_types::{
    EncodedTransaction, UiInstruction, UiMessage, UiParsedInstruction,
};
use std::{borrow::Cow, convert::identity, str::FromStr};
use uint::construct_uint;

use crate::{
    api::solana_rpc::Transaction,
    raydium::{
        event_processors::RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID,
        models::{
            AccountFlag, AmmInfo, AmmKeys, Market, MarketKeys, MarketState, MarketStateV2, Pool,
        },
    },
};

pub const AUTHORITY_AMM: &[u8] = b"amm authority";
pub const ACCOUNT_HEAD_PADDING: &[u8; 5] = b"serum";
pub const ACCOUNT_TAIL_PADDING: &[u8; 7] = b"padding";
pub const TEN_THOUSAND: u64 = 10000;

use super::{EventProcessor, WSOL};

//#[derive(Copy, Clone, Debug, Eq, PartialEq)]
//#[repr(u64)]
//pub enum SwapDirection {
//    /// Input token pc, output token coin
//    PC2Coin = 1u64,
//    /// Input token coin, output token pc
//    Coin2PC = 2u64,
//}
//
//construct_uint! {
//    pub struct U128(2);
//}

impl EventProcessor {
    pub async fn get_pool_from_create_transaction(&self, signature: &str) -> anyhow::Result<Pool> {
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
                                //fees: None,
                                //state_data: None,
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

                            return Ok(pool);
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Pool not found"))
    }

    pub async fn buy_new_pool(
        &self,
        owner: &Keypair,
        target: Pubkey,
        amount: u64,
        signature: &str,
        simulate_only: bool,
    ) -> anyhow::Result<()> {
        println!("RAYDIUM - Signature: {:#?}", &signature);

        let pool = self.get_pool_from_create_transaction(signature).await?;

        if pool.amm.amm_coin_mint != target || pool.amm.amm_pc_mint != target {
            return anyhow::Result::Err(anyhow::anyhow!("Target not found in pool creation"));
        }

        self.buy(owner, target, pool, amount, simulate_only).await?;

        Ok(())
    }

    async fn buy(
        &self,
        owner: &Keypair,
        target: Pubkey,
        pool: Pool,
        amount: u64,
        simulate_only: bool,
    ) -> anyhow::Result<()> {
        let token_mint_input = WSOL;
        let token_mint_output = target;
        let slippage_bps = 1000_u64;
        let amount_specified_is_input = true;
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

        let market_keys = self.get_market_keys(&pool).await?;
        //let (direction, coin_to_pc) = if token_mint_input == pool.amm.amm_coin_mint
        //    && token_mint_output == pool.amm.amm_pc_mint
        //{
        //    (SwapDirection::Coin2PC, true)
        //} else {
        //    (SwapDirection::PC2Coin, false)
        //};
        //
        //let (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount) =
        //    Self::calc_total_without_take_pnl_no_orderbook(
        //        pool.initial_pc_balance,
        //        pool.initial_coin_balance,
        //        &pool.amm,
        //    )?;
        //
        //let (min_output_amount, other_min_output_amount) = Self::swap_with_slippage(
        //    amm_pool_pc_vault_amount,
        //    amm_pool_coin_vault_amount,
        //    //pool.amm.fees.swap_fee_numerator,
        //    0,
        //    //pool.amm.fees.swap_fee_denominator,
        //    0,
        //    direction,
        //    amount,
        //    amount_specified_is_input,
        //    slippage_bps,
        //)?;
        //
        //println!("Min Output Amount: {:#?}", &min_output_amount);
        //println!("Other Min Output Amount: {:#?}", &other_min_output_amount);
        //let (amm_keys, market_keys) = self.get_serum_quote(&pool).await?;
        //
        //    //println!("AMM Keys: {:#?}", &amm_keys);
        //    //println!("Market Keys: {:#?}", &market_keys);
        //
        //    let amount_in: u64 = 143594511; //1_000_000;

        let min_output_amount = 900_000_u64;
        let instruction_tag = 9u8; // "Swap" tag, https://github.com/reactive-biscuit/raydium-amm/blob/ae039d21cd49ef670d76b3a1cf5485ae0213dc5e/program/src/instruction.rs#L487
        let mut swap_data = vec![instruction_tag];
        swap_data.extend_from_slice(&amount.to_le_bytes());
        swap_data.extend_from_slice(&min_output_amount.to_le_bytes());
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

    //async fn get_serum_quote(&self, pool: &Pool) -> anyhow::Result<MarketKeys> {
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

    //let market_keys = self.get_market_keys(pool).await?;
    //Ok(market_keys)
    //}

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
                                //fees: None,
                                //state_data: None,
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

    pub(crate) async fn get_market_keys(&self, pool: &Pool) -> anyhow::Result<MarketKeys> {
        let account_data = self.solana_api.get_account_data(&pool.amm.market).await?;
        let words = Self::remove_dex_account_padding(&account_data).map_err(anyhow::Error::msg)?;

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

        println!("Market State: {:#?}", &market_state);

        let vault_signer_key = Self::gen_vault_signer_key(
            market_state.vault_signer_nonce,
            &pool.amm.market,
            &pool.amm.market_program,
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

        Ok(market_keys)
    }

    fn remove_dex_account_padding<'a>(data: &'a [u8]) -> Result<Cow<'a, [u64]>, String> {
        let head = &data[..ACCOUNT_HEAD_PADDING.len()];
        if data.len() < ACCOUNT_HEAD_PADDING.len() + ACCOUNT_TAIL_PADDING.len() {
            return Err(format!(
                "dex account length {} is too small to contain valid padding",
                data.len()
            ));
        }

        if head != ACCOUNT_HEAD_PADDING {
            return Err("dex account head padding mismatch".to_string());
        }

        let tail = &data[data.len() - ACCOUNT_TAIL_PADDING.len()..];
        if tail != ACCOUNT_TAIL_PADDING {
            return Err("dex account tail padding mismatch".to_string());
        }

        let inner_data_range =
            ACCOUNT_HEAD_PADDING.len()..(data.len() - ACCOUNT_TAIL_PADDING.len());

        let inner = &data[inner_data_range];
        let words: Cow<'a, [u64]> = match transmute_many_pedantic::<u64>(inner) {
            Ok(word_slice) => Cow::Borrowed(word_slice),
            Err(transmute_error) => {
                let word_vec = transmute_error.copy().map_err(|e| e.to_string())?;
                Cow::Owned(word_vec)
            }
        };

        Ok(words)
    }

    #[inline]
    fn gen_vault_signer_key(
        nonce: u64,
        market: &Pubkey,
        program_id: &Pubkey,
    ) -> Result<Pubkey, ProgramError> {
        let seeds = Self::gen_vault_signer_seeds(&nonce, market);
        Ok(Pubkey::create_program_address(&seeds, program_id)?)
    }

    fn gen_vault_signer_seeds<'a>(nonce: &'a u64, market: &'a Pubkey) -> [&'a [u8]; 2] {
        [market.as_ref(), bytes_of(nonce)]
    }

    //fn swap_with_slippage(
    //    pc_vault_amount: u64,
    //    coin_vault_amount: u64,
    //    swap_fee_numerator: u64,
    //    swap_fee_denominator: u64,
    //    swap_direction: SwapDirection,
    //    amount_specified: u64,
    //    swap_base_in: bool,
    //    slippage_bps: u64,
    //) -> anyhow::Result<(u64, u64)> {
    //    let other_amount_threshold = Self::swap_exact_amount(
    //        pc_vault_amount,
    //        coin_vault_amount,
    //        swap_fee_numerator,
    //        swap_fee_denominator,
    //        swap_direction,
    //        amount_specified,
    //        swap_base_in,
    //    )?;
    //
    //    let quote = other_amount_threshold;
    //    let other_amount_threshold = if swap_base_in {
    //        // min out
    //        Self::min_amount_with_slippage(other_amount_threshold, slippage_bps)
    //    } else {
    //        // max in
    //        Self::max_amount_with_slippage(other_amount_threshold, slippage_bps)
    //    };
    //
    //    Ok((quote, other_amount_threshold))
    //
    //    //let other_amount_threshold = if swap_base_in {
    //    //    // min out
    //    //    Self::min_amount_with_slippage(other_amount_threshold, slippage_bps)
    //    //} else {
    //    //    // max in
    //    //    Self::max_amount_with_slippage(other_amount_threshold, slippage_bps)
    //    //};
    //    //Ok(other_amount_threshold)
    //}
    //
    //fn swap_exact_amount(
    //    pc_vault_amount: u64,
    //    coin_vault_amount: u64,
    //    swap_fee_numerator: u64,
    //    swap_fee_denominator: u64,
    //    swap_direction: SwapDirection,
    //    amount_specified: u64,
    //    swap_base_in: bool,
    //) -> anyhow::Result<u64> {
    //    let other_amount_threshold = if swap_base_in {
    //        let swap_fee = U128::from(amount_specified)
    //            .checked_mul(swap_fee_numerator.into())
    //            .unwrap()
    //            .checked_ceil_div(swap_fee_denominator.into())
    //            .unwrap()
    //            .0;
    //        let swap_in_after_deduct_fee =
    //            U128::from(amount_specified).checked_sub(swap_fee).unwrap();
    //
    //        Self::swap_token_amount_base_in(
    //            swap_in_after_deduct_fee,
    //            pc_vault_amount.into(),
    //            coin_vault_amount.into(),
    //            swap_direction,
    //        )
    //        .as_u64()
    //    } else {
    //        let swap_in_before_add_fee = Self::swap_token_amount_base_out(
    //            amount_specified.into(),
    //            pc_vault_amount.into(),
    //            coin_vault_amount.into(),
    //            swap_direction,
    //        );
    //
    //        swap_in_before_add_fee
    //            .checked_mul(swap_fee_denominator.into())
    //            .unwrap()
    //            .checked_ceil_div(
    //                (swap_fee_denominator
    //                    .checked_sub(swap_fee_numerator)
    //                    .unwrap())
    //                .into(),
    //            )
    //            .unwrap()
    //            .0
    //            .as_u64()
    //    };
    //
    //    Ok(other_amount_threshold)
    //}
    //
    //fn max_amount_with_slippage(input_amount: u64, slippage_bps: u64) -> u64 {
    //    input_amount
    //        .checked_mul(slippage_bps.checked_add(TEN_THOUSAND).unwrap())
    //        .unwrap()
    //        .checked_div(TEN_THOUSAND)
    //        .unwrap()
    //}
    //
    //fn min_amount_with_slippage(input_amount: u64, slippage_bps: u64) -> u64 {
    //    input_amount
    //        .checked_mul(TEN_THOUSAND.checked_sub(slippage_bps).unwrap())
    //        .unwrap()
    //        .checked_div(TEN_THOUSAND)
    //        .unwrap()
    //}
    //
    //pub fn swap_token_amount_base_in(
    //    amount_in: U128,
    //    total_pc_without_take_pnl: U128,
    //    total_coin_without_take_pnl: U128,
    //    swap_direction: SwapDirection,
    //) -> U128 {
    //    match swap_direction {
    //        SwapDirection::Coin2PC => {
    //            // (x + delta_x) * (y + delta_y) = x * y
    //            // (coin + amount_in) * (pc - amount_out) = coin * pc
    //            // => amount_out = pc - coin * pc / (coin + amount_in)
    //            // => amount_out = ((pc * coin + pc * amount_in) - coin * pc) / (coin + amount_in)
    //            // => amount_out =  pc * amount_in / (coin + amount_in)
    //            let denominator = total_coin_without_take_pnl.checked_add(amount_in).unwrap();
    //            total_pc_without_take_pnl
    //                .checked_mul(amount_in)
    //                .unwrap()
    //                .checked_div(denominator)
    //                .unwrap()
    //        }
    //        SwapDirection::PC2Coin => {
    //            // (x + delta_x) * (y + delta_y) = x * y
    //            // (pc + amount_in) * (coin - amount_out) = coin * pc
    //            // => amount_out = coin - coin * pc / (pc + amount_in)
    //            // => amount_out = (coin * pc + coin * amount_in - coin * pc) / (pc + amount_in)
    //            // => amount_out = coin * amount_in / (pc + amount_in)
    //            let denominator = total_pc_without_take_pnl.checked_add(amount_in).unwrap();
    //            total_coin_without_take_pnl
    //                .checked_mul(amount_in)
    //                .unwrap()
    //                .checked_div(denominator)
    //                .unwrap()
    //        }
    //    }
    //}
    //
    //pub fn swap_token_amount_base_out(
    //    amount_out: U128,
    //    total_pc_without_take_pnl: U128,
    //    total_coin_without_take_pnl: U128,
    //    swap_direction: SwapDirection,
    //) -> U128 {
    //    match swap_direction {
    //        SwapDirection::Coin2PC => {
    //            // (x + delta_x) * (y + delta_y) = x * y
    //            // (coin + amount_in) * (pc - amount_out) = coin * pc
    //            // => amount_in = coin * pc / (pc - amount_out) - coin
    //            // => amount_in = (coin * pc - pc * coin + amount_out * coin) / (pc - amount_out)
    //            // => amount_in = (amount_out * coin) / (pc - amount_out)
    //            let denominator = total_pc_without_take_pnl.checked_sub(amount_out).unwrap();
    //            total_coin_without_take_pnl
    //                .checked_mul(amount_out)
    //                .unwrap()
    //                .checked_ceil_div(denominator)
    //                .unwrap()
    //                .0
    //        }
    //        SwapDirection::PC2Coin => {
    //            // (x + delta_x) * (y + delta_y) = x * y
    //            // (pc + amount_in) * (coin - amount_out) = coin * pc
    //            // => amount_out = coin - coin * pc / (pc + amount_in)
    //            // => amount_out = (coin * pc + coin * amount_in - coin * pc) / (pc + amount_in)
    //            // => amount_out = coin * amount_in / (pc + amount_in)
    //
    //            // => amount_in = coin * pc / (coin - amount_out) - pc
    //            // => amount_in = (coin * pc - pc * coin + pc * amount_out) / (coin - amount_out)
    //            // => amount_in = (pc * amount_out) / (coin - amount_out)
    //            let denominator = total_coin_without_take_pnl.checked_sub(amount_out).unwrap();
    //            total_pc_without_take_pnl
    //                .checked_mul(amount_out)
    //                .unwrap()
    //                .checked_ceil_div(denominator)
    //                .unwrap()
    //                .0
    //        }
    //    }
    //}
    //
    //fn calc_total_without_take_pnl_no_orderbook<'a>(
    //    pc_amount: u64,
    //    coin_amount: u64,
    //    amm: &'a AmmKeys,
    //) -> anyhow::Result<(u64, u64)> {
    //    let total_pc_without_take_pnl = pc_amount
    //        //.checked_sub(amm.state_data.need_take_pnl_pc)
    //        .checked_sub(0)
    //        .with_context(|| "Failed to subtract take pnl pc")?;
    //
    //    let total_coin_without_take_pnl = coin_amount
    //        //.checked_sub(amm.state_data.need_take_pnl_coin)
    //        .checked_sub(0)
    //        .with_context(|| "Failed to subtract take pnl coin")?;
    //
    //    Ok((total_pc_without_take_pnl, total_coin_without_take_pnl))
    //}
}

//pub trait CheckedCeilDiv: Sized {
//    /// Perform ceiling division
//    fn checked_ceil_div(&self, rhs: Self) -> Option<(Self, Self)>;
//}

//impl CheckedCeilDiv for u128 {
//    fn checked_ceil_div(&self, mut rhs: Self) -> Option<(Self, Self)> {
//        let mut quotient = self.checked_div(rhs)?;
//        // Avoid dividing a small number by a big one and returning 1, and instead
//        // fail.
//        if quotient == 0 {
//            // return None;
//            if self.checked_mul(2_u128)? >= rhs {
//                return Some((1, 0));
//            } else {
//                return Some((0, 0));
//            }
//        }
//
//        // Ceiling the destination amount if there's any remainder, which will
//        // almost always be the case.
//        let remainder = self.checked_rem(rhs)?;
//        if remainder > 0 {
//            quotient = quotient.checked_add(1)?;
//            // calculate the minimum amount needed to get the dividend amount to
//            // avoid truncating too much
//            rhs = self.checked_div(quotient)?;
//            let remainder = self.checked_rem(quotient)?;
//            if remainder > 0 {
//                rhs = rhs.checked_add(1)?;
//            }
//        }
//        Some((quotient, rhs))
//    }
//}
//
//impl CheckedCeilDiv for U128 {
//    fn checked_ceil_div(&self, mut rhs: Self) -> Option<(Self, Self)> {
//        let mut quotient = self.checked_div(rhs)?;
//        // Avoid dividing a small number by a big one and returning 1, and instead
//        // fail.
//        let zero = U128::from(0);
//        let one = U128::from(1);
//        if quotient.is_zero() {
//            // return None;
//            if self.checked_mul(U128::from(2))? >= rhs {
//                return Some((one, zero));
//            } else {
//                return Some((zero, zero));
//            }
//        }
//
//        // Ceiling the destination amount if there's any remainder, which will
//        // almost always be the case.
//        let remainder = self.checked_rem(rhs)?;
//        if remainder > zero {
//            quotient = quotient.checked_add(one)?;
//            // calculate the minimum amount needed to get the dividend amount to
//            // avoid truncating too much
//            rhs = self.checked_div(quotient)?;
//            let remainder = self.checked_rem(quotient)?;
//            if remainder > zero {
//                rhs = rhs.checked_add(one)?;
//            }
//        }
//        Some((quotient, rhs))
//    }
//}
