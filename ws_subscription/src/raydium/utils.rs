use std::{borrow::Cow, error::Error};

use bytemuck::bytes_of;
use safe_transmute::transmute_many_pedantic;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, program_error::ProgramError, pubkey::Pubkey,
};

pub const AUTHORITY_AMM: &[u8] = b"amm authority";
pub const ACCOUNT_HEAD_PADDING: &[u8; 5] = b"serum";
pub const ACCOUNT_TAIL_PADDING: &[u8; 7] = b"padding";

pub async fn get_account<T>(
    client: &RpcClient,
    account: &Pubkey,
) -> Result<Option<T>, Box<dyn Error>>
where
    T: Clone,
{
    let response = client
        .get_account_with_commitment(account, CommitmentConfig::processed())
        .await?;

    if let Some(account) = response.value {
        let account_data = account.data.as_slice();
        let ret = unsafe { &*(&account_data[0] as *const u8 as *const T) };
        Ok(Some(ret.clone()))
    } else {
        Ok(None)
    }
}

pub fn compute_amm_authority_id(
    program_id: &Pubkey,
    nonce: u8,
) -> Result<Pubkey, Box<(dyn Error)>> {
    let result = Pubkey::create_program_address(&[AUTHORITY_AMM, &[nonce]], program_id)?;

    Ok(result)
}

pub fn remove_dex_account_padding<'a>(data: &'a [u8]) -> Result<Cow<'a, [u64]>, String> {
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
    let inner_data_range = ACCOUNT_HEAD_PADDING.len()..(data.len() - ACCOUNT_TAIL_PADDING.len());
    let inner: &'a [u8] = &data[inner_data_range];
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
pub fn gen_vault_signer_key(
    nonce: u64,
    market: &Pubkey,
    program_id: &Pubkey,
) -> Result<Pubkey, ProgramError> {
    let seeds = gen_vault_signer_seeds(&nonce, market);
    Ok(Pubkey::create_program_address(&seeds, program_id)?)
}

fn gen_vault_signer_seeds<'a>(nonce: &'a u64, market: &'a Pubkey) -> [&'a [u8]; 2] {
    [market.as_ref(), bytes_of(nonce)]
}
