//use futures::StreamExt;
//use safe_transmute::{transmute_one_pedantic, transmute_one_to_bytes, transmute_to_bytes};
//use serde_json::{json, Value};
//use solana_client::{
//    client_error::reqwest,
//    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
//    rpc_config::{RpcTransactionConfig, RpcTransactionLogsConfig, RpcTransactionLogsFilter},
//};
//use solana_sdk::{
//    commitment_config::{CommitmentConfig, CommitmentLevel},
//    compute_budget::ComputeBudgetInstruction,
//    instruction::{AccountMeta, Instruction},
//    message::{Message, VersionedMessage},
//    pubkey::Pubkey,
//    signature::{Keypair, Signature},
//    signer::{EncodableKey, Signer},
//    transaction::{Transaction, VersionedTransaction},
//};
//use solana_transaction_status_client_types::{
//    EncodedTransaction, UiInstruction, UiMessage, UiParsedInstruction, UiTransactionEncoding,
//    UiTransactionStatusMeta,
//};
//use std::{convert::identity, error::Error, str::FromStr};
//
//use crate::raydium::{
//    models::{AccountFlag, Market, MarketState, MarketStateV2},
//    utils::{gen_vault_signer_key, remove_dex_account_padding},
//};
//
//use super::{
//    event_processors::EventProcessor,
//    models::{AmmInfo, AmmKeys, MarketKeys},
//    utils::{compute_amm_authority_id, get_account},
//};
//
//const RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID: Pubkey =
//    solana_sdk::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
//const TOKEN_PROGRAM: Pubkey = solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
//const SERUM_PROGRAM: Pubkey = solana_sdk::pubkey!("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX");

use super::event_processors::EventProcessor;

pub async fn execute_demo(ws_url: &str, rpc_url: &str) -> anyhow::Result<()> {
    let signature =
        "2LbMvKeJJbaUfYxpbgYw5LTQ3P1qviMyV8d72BnkyqkwdJ8T5k7knyUE2dPFUxqLwFAoGymmDcm3t73mLEwxhued";
    let raydium_processor = EventProcessor::new(rpc_url, ws_url).await?;
    //raydium_processor.execute().await?;
    raydium_processor.process_new_pool(signature).await?;

    Ok(())
}

//pub async fn test_pool_created(rpc_url: &str) -> Result<(), Box<dyn std::error::Error>> {
//    let rpc_client = RpcClient::new(rpc_url.to_string());
//    let signature =
//        "55HKWEgpP1dZwejMukwP4puJP469uMMivuwwcMwqJcKRm1iECL2SH3mgJz72neaGVqYgNC55J6s2m6ig26C7XvT3";
//
//    process_new_pool_detected(signature.to_string(), &rpc_client).await
//}
//
//pub async fn test_swap_detected(rpc_url: &str) -> Result<(), Box<dyn std::error::Error>> {
//    let rpc_client = RpcClient::new(rpc_url.to_string());
//    let signature =
//        "2QuUoNKiCAopvA6KC64sNrfVETmrMQXv6vRHxrwMdc2pB3RnoFCqdbANQMZAxVgKHM6ny3NeNmcQnTgs3SYCfuiq";
//
//    process_new_swap_detected(signature.to_string(), &rpc_client).await
//}
//
//pub async fn test_swap_exact_input(
//    rpc_url: &str,
//    keypair_file_path: &str,
//) -> Result<(), Box<dyn std::error::Error>> {
//    let rpc_client = RpcClient::new(rpc_url.to_string());
//
//    swap_exact_input(&rpc_client, keypair_file_path).await
//}
//
//pub async fn test_swap_via_api(
//    rpc_url: &str,
//    keypait_file_path: &str,
//) -> Result<(), Box<dyn std::error::Error>> {
//    let rpc_client = RpcClient::new(rpc_url.to_string());
//
//    let token_mint_input = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");
//    let token_mint_output = solana_sdk::pubkey!("7uJrMsDN2Wxdc3VAq1iK9N5AHaTA7wUpbm1wqRonpump");
//
//    let token_account_input =
//        Pubkey::from_str("GKNaPCWkQfg8KK8gxtnCrVWsgLPUj9AhoywTQ7yX9GBE").unwrap(); //7uJrMsDN2Wxdc3VAq1iK9N5AHaTA7wUpbm1wqRonpump
//    let token_account_output =
//        Pubkey::from_str("4oSZPX4QJkpy12VaGuJjRiMaFxAMFdKZpJznSbEFm61h").unwrap();
//
//    let owner = Keypair::read_from_file(keypait_file_path).expect("Error parsing private key");
//
//    let fee = get_fee().await.unwrap();
//
//    let quote = compute_swap(
//        &token_mint_input.to_string(),
//        &token_mint_output.to_string(),
//        "1000000",
//    )
//    .await
//    .unwrap();
//
//    let transaction = get_transaction(
//        &owner.pubkey(),
//        &token_account_input,
//        &token_account_output,
//        fee,
//        quote,
//    )
//    .await
//    .unwrap();
//
//    let transaction_bytes = base64::decode(transaction).unwrap();
//
//    let transaction: VersionedTransaction = bincode::deserialize(&transaction_bytes).unwrap();
//    let signers: [&dyn Signer; 1] = [&owner];
//    let transaction = VersionedTransaction::try_new(transaction.message, &signers).unwrap();
//
//    let tx_signature = rpc_client
//        .send_and_confirm_transaction(&transaction)
//        .await
//        .unwrap();
//
//    println!("Transaction signature: {:?}", tx_signature);
//
//    Ok(())
//}
//
//async fn process_new_pool_detected(
//    signature: String,
//    rpc_client: &RpcClient,
//) -> Result<(), Box<dyn std::error::Error>> {
//    let (metadata, transaction) = get_transaction_data(&signature, rpc_client).await?;
//
//    println!("\nMetadata: {:#?}\n", &metadata);
//    println!("\nTransaction: {:#?}\n", &transaction);
//
//    if let EncodedTransaction::Json(ui_transaction) = transaction {
//        if let UiMessage::Parsed(ui_parsed_message) = ui_transaction.message {
//            for instruction in ui_parsed_message.instructions {
//                if let UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(
//                    parsed_instruction,
//                )) = instruction
//                {
//                    if parsed_instruction.program_id
//                        == RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID.to_string()
//                    {
//                        println!("------------ New Pool Detected ------------");
//                        println!("    Tx Signature: {:#?}", &signature);
//                        let amm = &parsed_instruction.accounts[4];
//                        let amm_authority = &parsed_instruction.accounts[5];
//                        let amm_open_orders = &parsed_instruction.accounts[6];
//                        let amm_lp_mint = &parsed_instruction.accounts[7];
//                        let amm_coin_mint = &parsed_instruction.accounts[8];
//                        let amm_coin_vault = &parsed_instruction.accounts[10];
//                        let amm_pc_mint = &parsed_instruction.accounts[9];
//                        let amm_pc_vault = &parsed_instruction.accounts[11];
//                        let amm_target = &parsed_instruction.accounts[13];
//                        let market_program = &parsed_instruction.accounts[15];
//                        let market = &parsed_instruction.accounts[16];
//
//                        let amm_keys = AmmKeys {
//                            amm_pool: Pubkey::from_str(amm).unwrap(),
//                            amm_coin_mint: Pubkey::from_str(amm_coin_mint).unwrap(),
//                            amm_pc_mint: Pubkey::from_str(amm_pc_mint).unwrap(),
//                            amm_authority: Pubkey::from_str(amm_authority).unwrap(),
//                            amm_target: Pubkey::from_str(amm_target).unwrap(),
//                            amm_coin_vault: Pubkey::from_str(amm_coin_vault).unwrap(),
//                            amm_pc_vault: Pubkey::from_str(amm_pc_vault).unwrap(),
//                            amm_lp_mint: Pubkey::from_str(amm_lp_mint).unwrap(),
//                            amm_open_order: Pubkey::from_str(amm_open_orders).unwrap(),
//                            market_program: Pubkey::from_str(market_program).unwrap(),
//                            market: Pubkey::from_str(market).unwrap(),
//                            nonce: 0,
//                        };
//
//                        let balances = rpc_client
//                            .get_token_account_balance(&Pubkey::from_str(amm_coin_vault).unwrap())
//                            .await?;
//                        println!("    Token A Balance: {:#?}", balances);
//
//                        render_pool_info(
//                            amm,
//                            amm_coin_mint,
//                            amm_coin_vault,
//                            amm_pc_vault,
//                            amm_pc_mint,
//                            &metadata,
//                        )
//                        .await?;
//                        println!("-------------------------------------------");
//                    }
//                }
//            }
//        }
//    }
//
//    Ok(())
//}
//
//async fn process_new_swap_detected(
//    signature: String,
//    rpc_client: &RpcClient,
//) -> Result<(), Box<dyn std::error::Error>> {
//    let (metadata, transaction) = get_transaction_data(&signature, rpc_client).await?;
//
//    println!("------------ New Swap Detected ------------");
//    println!("    Tx Signature: {:#?}", &signature);
//
//    let mut token_in = String::new();
//    let mut token_out = String::new();
//    let mut amount_in = "0".to_owned();
//    let mut amount_out = "0".to_owned();
//
//    if let EncodedTransaction::Json(ui_transaction) = transaction {
//        if let UiMessage::Parsed(ui_parsed_message) = ui_transaction.message {
//            for instruction in ui_parsed_message.instructions {
//                if let UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(
//                    parsed_instruction,
//                )) = instruction
//                {
//                    if parsed_instruction.program_id
//                        == RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID.to_string()
//                    {
//                        let token_account_a = &parsed_instruction.accounts[5];
//                        let token_account_b = &parsed_instruction.accounts[6];
//
//                        let inner_instructions = metadata.inner_instructions.clone().unwrap();
//                        for inner_instruction in inner_instructions {
//                            for instruction in inner_instruction.instructions {
//                                if let UiInstruction::Parsed(UiParsedInstruction::Parsed(
//                                    ui_parsed_instruction,
//                                )) = instruction
//                                {
//                                    if ui_parsed_instruction.parsed.get("type").unwrap()
//                                        == "transfer"
//                                    {
//                                        let info =
//                                            ui_parsed_instruction.parsed.get("info").unwrap();
//
//                                        if info.get("amount").is_none() {
//                                            continue;
//                                        }
//                                        let source = info.get("source").unwrap();
//                                        let destination = info.get("destination").unwrap();
//                                        let amount = info.get("amount").unwrap();
//
//                                        //println!("    Source: {:#?}", source);
//                                        //println!("    Destination: {:#?}", destination);
//                                        //println!("    Amount: {:#?}", amount.as_str().unwrap());
//
//                                        if token_account_a == source {
//                                            token_in = token_account_a.to_owned();
//                                            amount_out = amount.as_str().unwrap().to_owned();
//                                        } else if token_account_b == source {
//                                            token_in = token_account_b.to_owned();
//                                            amount_out = amount.as_str().unwrap().to_owned();
//                                        } else if token_account_a == destination {
//                                            token_out = token_account_a.to_owned();
//                                            amount_in = amount.as_str().unwrap().to_owned();
//                                        } else if token_account_b == destination {
//                                            token_out = token_account_b.to_owned();
//                                            amount_in = amount.as_str().unwrap().to_owned();
//                                        }
//                                    }
//                                }
//                            }
//                        }
//                    }
//                }
//            }
//        }
//    }
//
//    println!("    Token Account In: {:#?}", token_in);
//    println!("    Amount In: {:#?}", amount_in);
//    println!("    Token Account Out: {:#?}", token_out);
//    println!("    Amount Out: {:#?}", amount_out);
//    println!("-------------------------------------------");
//
//    Ok(())
//}
//
//async fn swap_exact_input(
//    rpc_client: &RpcClient,
//    keypair_file_path: &str,
//) -> Result<(), Box<dyn std::error::Error>> {
//    let token_mint_input = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");
//    let token_mint_output = solana_sdk::pubkey!("7uJrMsDN2Wxdc3VAq1iK9N5AHaTA7wUpbm1wqRonpump");
//
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
//    let (amm_keys, market_keys) = get_serum_quote(rpc_client, &amm_keys.amm_pool)
//        .await
//        .unwrap();
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
//}
//
//async fn render_pool_info(
//    amm: &str,
//    token_address_a: &str,
//    token_account_a: &str,
//    token_account_b: &str,
//    token_address_b: &str,
//    metadata: &UiTransactionStatusMeta,
//) -> Result<(), Box<dyn std::error::Error>> {
//    let post_token_balances = metadata.post_token_balances.clone().unwrap();
//    let initial_token_balance_a = &post_token_balances
//        .iter()
//        .find(|x| x.mint == token_address_a)
//        .unwrap()
//        .ui_token_amount
//        .amount;
//
//    let initial_token_balance_b = &post_token_balances
//        .iter()
//        .find(|x| x.mint == token_address_b)
//        .unwrap()
//        .ui_token_amount
//        .amount;
//
//    println!("    AMM");
//    println!("        Address: {:#?}", amm);
//    println!("    Token A");
//    println!("        Address: {:#?}", token_address_a);
//    println!("        Account: {:#?}", token_account_a);
//    println!(
//        "        Initial Pool Balance: {:#?}",
//        initial_token_balance_a
//    );
//    println!();
//    println!("    Token B");
//    println!("        Address: {:#?}", token_address_b);
//    println!("        Account: {:#?}", token_account_b);
//    println!(
//        "        Initial Pool Balance: {:#?}",
//        initial_token_balance_b
//    );
//
//    Ok(())
//}
//
//async fn get_transaction_data(
//    signature: &str,
//    rpc_client: &RpcClient,
//) -> Result<(UiTransactionStatusMeta, EncodedTransaction), Box<dyn std::error::Error>> {
//    let signature = Signature::from_str(&signature).unwrap();
//
//    let transaction = rpc_client
//        .get_transaction_with_config(
//            &signature,
//            RpcTransactionConfig {
//                encoding: Some(UiTransactionEncoding::JsonParsed),
//                commitment: Some(CommitmentConfig {
//                    commitment: CommitmentLevel::Confirmed,
//                }),
//                max_supported_transaction_version: Some(0),
//            },
//        )
//        .await
//        .unwrap();
//
//    let metadata = transaction.transaction.meta.unwrap();
//    let transaction = transaction.transaction.transaction;
//
//    Ok((metadata, transaction))
//}
//
//pub async fn get_serum_quote_via_raydium_api(
//    input_token_mint: &str,
//    output_token_mint: &str,
//) -> Result<(AmmKeys, MarketKeys), Box<dyn Error>> {
//    let base_url = "https://api-v3.raydium.io";
//    let url = format!(
//        "{}/pools/info/mint?mint1={}&mint2={}&poolType={}&poolSortField={}&sortType={}&pageSize={}&page={}",
//        &base_url, input_token_mint, output_token_mint, "standard", "liquidity", "desc", 100, 1,
//    );
//
//    let result = reqwest::get(url).await?;
//    let response = result.json::<serde_json::Value>().await?;
//
//    let data = response.get("data").unwrap();
//    let data = data.get("data").unwrap();
//
//    let pool = data.get(0).unwrap();
//    let pool_id = pool.get("id").unwrap().as_str().unwrap();
//
//    let (amm_keys, market_keys) = get_pool_keys(vec![pool_id]).await?;
//
//    Ok((amm_keys, market_keys))
//}
//
//async fn get_serum_quote(
//    rpc_client: &RpcClient,
//    amm_pool: &Pubkey,
//) -> Result<(AmmKeys, MarketKeys), Box<dyn Error>> {
//    let amm_info = get_account::<AmmInfo>(rpc_client, amm_pool)
//        .await
//        .unwrap()
//        .unwrap();
//
//    let amm_keys = AmmKeys {
//        amm_pool: *amm_pool,
//        amm_target: amm_info.target_orders,
//        amm_coin_vault: amm_info.coin_vault,
//        amm_pc_vault: amm_info.pc_vault,
//        amm_lp_mint: amm_info.lp_mint,
//        amm_open_order: amm_info.open_orders,
//        amm_coin_mint: amm_info.coin_vault_mint,
//        amm_pc_mint: amm_info.pc_vault_mint,
//        amm_authority: compute_amm_authority_id(
//            &RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM_ID,
//            amm_info.nonce as u8,
//        )?,
//        market: amm_info.market,
//        market_program: amm_info.market_program,
//        nonce: amm_info.nonce as u8,
//    };
//
//    let account_data = rpc_client.get_account_data(&amm_keys.market).await?;
//    let words = remove_dex_account_padding(&account_data)?;
//
//    let market_state: MarketState = {
//        let account_flags = Market::account_flags(&account_data)?;
//        if account_flags.intersects(AccountFlag::Permissioned) {
//            let state = transmute_one_pedantic::<MarketStateV2>(transmute_to_bytes(&words))
//                .map_err(|e| e.without_src())?;
//            //state.check_flags(true)?;
//            state.inner
//        } else {
//            let state = transmute_one_pedantic::<MarketState>(transmute_to_bytes(&words))
//                .map_err(|e| e.without_src())?;
//            //state.check_flags(true)?;
//            state
//        }
//    };
//    let vault_signer_key = gen_vault_signer_key(
//        market_state.vault_signer_nonce,
//        &amm_keys.market,
//        &amm_keys.market_program,
//    )?;
//
//    let market_keys = MarketKeys {
//        event_queue: Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.event_q)))
//            .unwrap(),
//        bids: Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.bids))).unwrap(),
//        asks: Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.asks))).unwrap(),
//        coin_vault: Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.coin_vault)))
//            .unwrap(),
//        pc_vault: Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.pc_vault)))
//            .unwrap(),
//        vault_signer_key,
//    };
//
//    Ok((amm_keys, market_keys))
//}
//
//async fn get_pool_keys(pool_ids: Vec<&str>) -> Result<(AmmKeys, MarketKeys), Box<dyn Error>> {
//    let ids = pool_ids.join(",");
//    let base_url = "https://api-v3.raydium.io";
//    let url = format!("{}/pools/key/ids?ids={}", &base_url, ids);
//
//    let result = reqwest::get(url).await?;
//    let response = result.json::<serde_json::Value>().await?;
//
//    let data = response.get("data").unwrap();
//    let pool = data.get(0).unwrap();
//
//    let amm_pool = pool.get("id").unwrap().as_str().unwrap();
//    let amm_pool = Pubkey::from_str(amm_pool).unwrap();
//    let amm_coin_mint = pool.get("mintA").unwrap().get("address").unwrap();
//    let amm_coin_mint = Pubkey::from_str(amm_coin_mint.as_str().unwrap()).unwrap();
//    let amm_pc_mint = pool.get("mintB").unwrap().get("address").unwrap();
//    let amm_pc_mint = Pubkey::from_str(amm_pc_mint.as_str().unwrap()).unwrap();
//    let amm_authority = pool.get("authority").unwrap();
//    let amm_authority = Pubkey::from_str(amm_authority.as_str().unwrap()).unwrap();
//    let amm_target = pool.get("targetOrders").unwrap();
//    let amm_target = Pubkey::from_str(amm_target.as_str().unwrap()).unwrap();
//    let amm_coin_vault = pool.get("vault").unwrap().get("A").unwrap();
//    let amm_coin_vault = Pubkey::from_str(amm_coin_vault.as_str().unwrap()).unwrap();
//    let amm_pc_vault = pool.get("vault").unwrap().get("B").unwrap();
//    let amm_pc_vault = Pubkey::from_str(amm_pc_vault.as_str().unwrap()).unwrap();
//    let amm_lp_mint = pool.get("mintLp").unwrap().get("address").unwrap();
//    let amm_lp_mint = Pubkey::from_str(amm_lp_mint.as_str().unwrap()).unwrap();
//    let amm_open_order = pool.get("openOrders").unwrap();
//    let amm_open_order = Pubkey::from_str(amm_open_order.as_str().unwrap()).unwrap();
//    let market_program = pool.get("marketProgramId").unwrap();
//    let market_program = Pubkey::from_str(market_program.as_str().unwrap()).unwrap();
//    let market = pool.get("marketId").unwrap();
//    let market = Pubkey::from_str(market.as_str().unwrap()).unwrap();
//
//    let amm_keys = AmmKeys {
//        amm_pool,
//        amm_coin_mint,
//        amm_pc_mint,
//        amm_authority,
//        amm_target,
//        amm_coin_vault,
//        amm_pc_vault,
//        amm_lp_mint,
//        amm_open_order,
//        market_program,
//        market,
//        nonce: 0,
//    };
//
//    let event_queue = pool.get("marketEventQueue").unwrap();
//    let event_queue = Pubkey::from_str(event_queue.as_str().unwrap()).unwrap();
//    let bids = pool.get("marketBids").unwrap();
//    let bids = Pubkey::from_str(bids.as_str().unwrap()).unwrap();
//    let asks = pool.get("marketAsks").unwrap();
//    let asks = Pubkey::from_str(asks.as_str().unwrap()).unwrap();
//    let coin_vault = pool.get("marketBaseVault").unwrap();
//    let coin_vault = Pubkey::from_str(coin_vault.as_str().unwrap()).unwrap();
//    let pc_vault = pool.get("marketQuoteVault").unwrap();
//    let pc_vault = Pubkey::from_str(pc_vault.as_str().unwrap()).unwrap();
//    let vault_signer_key = pool.get("marketAuthority").unwrap();
//    let vault_signer_key = Pubkey::from_str(vault_signer_key.as_str().unwrap()).unwrap();
//
//    let market_keys = MarketKeys {
//        event_queue,
//        bids,
//        asks,
//        coin_vault,
//        pc_vault,
//        vault_signer_key,
//    };
//
//    Ok((amm_keys, market_keys))
//}
//
//async fn get_transaction(
//    owner: &Pubkey,
//    input_account: &Pubkey,
//    output_account: &Pubkey,
//    fee: u64,
//    swap_response: Value,
//) -> Result<String, Box<dyn std::error::Error>> {
//    let base_url = "https://transaction-v1.raydium.io";
//    let url = format!("{}/transaction/swap-base-in", &base_url,);
//    let client = reqwest::Client::new();
//
//    let result = client
//        .post(url)
//        .json(&json!({
//            "computeUnitPriceMicroLamports": fee.to_string(),
//            "swapResponse": swap_response,
//            "txVersion": "V0",
//            "wallet": owner.to_string(),
//            "wrapSol": false,
//            "unwrapSol": false,
//            "inputAccount": input_account.to_string(),
//            "outputAccount": output_account.to_string(),
//        }))
//        .send()
//        .await?;
//    let response = result.json::<serde_json::Value>().await?;
//
//    let data = response.get("data").unwrap();
//    let transaction = data.get(0).unwrap();
//    let transaction = transaction.get("transaction").unwrap();
//
//    Ok(transaction.as_str().unwrap().to_owned())
//}
//
//async fn compute_swap(
//    token_1: &str,
//    token_2: &str,
//    amount: &str,
//) -> Result<Value, Box<dyn std::error::Error>> {
//    let base_url = "https://transaction-v1.raydium.io";
//    let url = format!(
//        "{}/compute/swap-base-in?inputMint={}&outputMint={}&amount={}&slippageBps={}&txVersion={}",
//        &base_url, token_1, token_2, amount, "500", "V0",
//    );
//
//    let result = reqwest::get(url).await?;
//    let response = result.json::<serde_json::Value>().await?;
//
//    Ok(response)
//}
//
//async fn get_fee() -> Result<u64, Box<dyn std::error::Error>> {
//    let base_url = "https://api-v3.raydium.io";
//    let url = format!("{}/main/auto-fee", &base_url,);
//
//    let result = reqwest::get(url).await?;
//    let response = result.json::<serde_json::Value>().await?;
//
//    let data = response.get("data").unwrap();
//    let fee = data.get("default").unwrap().get("h").unwrap();
//
//    Ok(fee.as_u64().unwrap())
//}
