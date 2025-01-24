use std::cell::{Ref, RefMut};

use enumflags2::{bitflags, BitFlags};
use safe_transmute::TriviallyTransmutable;
use solana_sdk::pubkey::Pubkey;

use super::utils::ACCOUNT_HEAD_PADDING;

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct AmmInfo {
    /// Initialized status.
    pub status: u64,
    /// Nonce used in program address.
    /// The program address is created deterministically with the nonce,
    /// amm program id, and amm account pubkey.  This program address has
    /// authority over the amm's token coin account, token pc account, and pool
    /// token mint.
    pub nonce: u64,
    /// max order count
    pub max_order: u64,
    /// within this range, 5 => 5% range
    pub depth: u64,
    /// coin decimal
    pub coin_decimals: u64,
    /// pc decimal
    pub pc_decimals: u64,
    /// amm machine state
    pub state: u64,
    /// amm reset_flag
    pub reset_flag: u64,
    /// min size 1->0.000001
    pub min_size: u64,
    /// vol_max_cut_ratio numerator, sys_decimal_value as denominator
    pub vol_max_cut_ratio: u64,
    /// amount wave numerator, sys_decimal_value as denominator
    pub amount_wave: u64,
    /// coinLotSize 1 -> 0.000001
    pub coin_lot_size: u64,
    /// pcLotSize 1 -> 0.000001
    pub pc_lot_size: u64,
    /// min_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub min_price_multiplier: u64,
    /// max_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub max_price_multiplier: u64,
    /// system decimal value, used to normalize the value of coin and pc amount
    pub sys_decimal_value: u64,
    /// All fee information
    pub fees: Fees,
    /// Statistical data
    pub state_data: StateData,
    /// Coin vault
    pub coin_vault: Pubkey,
    /// Pc vault
    pub pc_vault: Pubkey,
    /// Coin vault mint
    pub coin_vault_mint: Pubkey,
    /// Pc vault mint
    pub pc_vault_mint: Pubkey,
    /// lp mint
    pub lp_mint: Pubkey,
    /// open_orders key
    pub open_orders: Pubkey,
    /// market key
    pub market: Pubkey,
    /// market program key
    pub market_program: Pubkey,
    /// target_orders key
    pub target_orders: Pubkey,
    /// padding
    pub padding1: [u64; 8],
    /// amm owner key
    pub amm_owner: Pubkey,
    /// pool lp amount
    pub lp_amount: u64,
    /// client order id
    pub client_order_id: u64,
    /// padding
    pub padding2: [u64; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StateData {
    /// delay to take pnl coin
    pub need_take_pnl_coin: u64,
    /// delay to take pnl pc
    pub need_take_pnl_pc: u64,
    /// total pnl pc
    pub total_pnl_pc: u64,
    /// total pnl coin
    pub total_pnl_coin: u64,
    /// ido pool open time
    pub pool_open_time: u64,
    /// padding for future updates
    //pub padding: [u64; 2],
    pub punish_pc_amount: u64,
    pub punish_coin_amount: u64,

    /// switch from orderbookonly to init
    pub orderbook_to_init_time: u64,

    /// swap coin in amount
    pub swap_coin_in_amount: u128,
    /// swap pc out amount
    pub swap_pc_out_amount: u128,
    /// charge pc as swap fee while swap pc to coin
    pub swap_acc_pc_fee: u64,

    /// swap pc in amount
    pub swap_pc_in_amount: u64,
    /// swap coin out amount
    pub swap_coin_out_amount: u128,
    /// charge coin as swap fee while swap coin to pc
    pub swap_acc_coin_fee: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fees {
    /// numerator of the min_separate
    pub min_separate_numerator: u64,
    /// denominator of the min_separate
    pub min_separate_denominator: u64,

    /// numerator of the fee
    pub trade_fee_numerator: u64,
    /// denominator of the fee
    /// and 'trade_fee_denominator' must be equal to 'min_separate_denominator'
    pub trade_fee_denominator: u64,

    /// numerator of the pnl
    pub pnl_numerator: u64,
    /// denominator of the pnl
    pub pnl_denominator: u64,

    /// numerator of the swap_fee
    pub swap_fee_numerator: u64,
    /// denominator of the swap_fee
    pub swap_fee_denominator: u64,
}

#[bitflags]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum AccountFlag {
    Initialized = 1u64 << 0,
    Market = 1u64 << 1,
    OpenOrders = 1u64 << 2,
    RequestQueue = 1u64 << 3,
    EventQueue = 1u64 << 4,
    Bids = 1u64 << 5,
    Asks = 1u64 << 6,
    Disabled = 1u64 << 7,
    Closed = 1u64 << 8,
    Permissioned = 1u64 << 9,
    CrankAuthorityRequired = 1u64 << 10,
}

pub enum Market<'a> {
    V1(RefMut<'a, MarketState>),
    V2(RefMut<'a, MarketStateV2>),
    V1Ref(Ref<'a, MarketState>),
    V2Ref(Ref<'a, MarketStateV2>),
}

impl Market<'_> {
    pub fn account_flags(account_data: &[u8]) -> anyhow::Result<BitFlags<AccountFlag>> {
        let start = ACCOUNT_HEAD_PADDING.len();
        let end = start + size_of::<AccountFlag>();
        assert!(account_data.len() >= end);

        let mut flag_bytes = [0u8; 8];
        flag_bytes.copy_from_slice(&account_data[start..end]);

        BitFlags::from_bits(u64::from_le_bytes(flag_bytes))
            .map_err(|e| anyhow::Error::msg(e.to_string()))
            .map(Into::into)
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(target_endian = "little", derive(Debug))]
#[repr(packed)]
pub struct MarketStateV2 {
    pub inner: MarketState,
    pub open_orders_authority: Pubkey,
    pub prune_authority: Pubkey,
    pub consume_events_authority: Pubkey,
    // Unused bytes for future upgrades.
    padding: [u8; 992],
}

unsafe impl TriviallyTransmutable for MarketStateV2 {}

impl MarketStateV2 {
    #[inline]
    pub fn check_flags(&self, allow_disabled: bool) -> Result<(), String> {
        let flags = BitFlags::from_bits(self.inner.account_flags).map_err(|e| e.to_string())?;

        let required_flags =
            AccountFlag::Initialized | AccountFlag::Market | AccountFlag::Permissioned;
        let required_crank_flags = required_flags | AccountFlag::CrankAuthorityRequired;

        if allow_disabled {
            let disabled_flags = required_flags | AccountFlag::Disabled;
            let disabled_crank_flags = required_crank_flags | AccountFlag::Disabled;
            if flags != required_flags
                && flags != required_crank_flags
                && flags != disabled_flags
                && flags != disabled_crank_flags
            {
                return Err("Invalid Market Flag".to_string());
            }
        } else if flags != required_flags && flags != required_crank_flags {
            return Err("Invalid Market Flag".to_string());
        }

        Ok(())
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(target_endian = "little", derive(Debug))]
#[repr(packed)]
pub struct MarketState {
    // 0
    pub account_flags: u64, // Initialized, Market

    // 1
    pub own_address: [u64; 4],

    // 5
    pub vault_signer_nonce: u64,
    // 6
    pub coin_mint: [u64; 4],
    // 10
    pub pc_mint: [u64; 4],

    // 14
    pub coin_vault: [u64; 4],
    // 18
    pub coin_deposits_total: u64,
    // 19
    pub coin_fees_accrued: u64,

    // 20
    pub pc_vault: [u64; 4],
    // 24
    pub pc_deposits_total: u64,
    // 25
    pub pc_fees_accrued: u64,

    // 26
    pub pc_dust_threshold: u64,

    // 27
    pub req_q: [u64; 4],
    // 31
    pub event_q: [u64; 4],

    // 35
    pub bids: [u64; 4],
    // 39
    pub asks: [u64; 4],

    // 43
    pub coin_lot_size: u64,
    // 44
    pub pc_lot_size: u64,

    // 45
    pub fee_rate_bps: u64,
    // 46
    pub referrer_rebates_accrued: u64,
}

unsafe impl TriviallyTransmutable for MarketState {}

impl MarketState {
    #[inline]
    pub fn check_flags(&self, allow_disabled: bool) -> Result<(), String> {
        let flags = BitFlags::from_bits(self.account_flags).map_err(|e| e.to_string())?;

        let required_flags =
            AccountFlag::Initialized | AccountFlag::Market | AccountFlag::Permissioned;
        let required_crank_flags = required_flags | AccountFlag::CrankAuthorityRequired;

        if allow_disabled {
            let disabled_flags = required_flags | AccountFlag::Disabled;
            let disabled_crank_flags = required_crank_flags | AccountFlag::Disabled;
            if flags != required_flags
                && flags != required_crank_flags
                && flags != disabled_flags
                && flags != disabled_crank_flags
            {
                return Err("Invalid Market Flag".to_string());
            }
        } else if flags != required_flags && flags != required_crank_flags {
            return Err("Invalid Market Flag".to_string());
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AmmKeys {
    pub amm_pool: Pubkey,
    pub amm_coin_mint: Pubkey,
    pub amm_pc_mint: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_target: Pubkey,
    pub amm_coin_vault: Pubkey,
    pub amm_pc_vault: Pubkey,
    pub amm_lp_mint: Pubkey,
    pub amm_open_order: Pubkey,
    pub market_program: Pubkey,
    pub market: Pubkey,
    pub nonce: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct MarketKeys {
    pub event_queue: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,
    pub vault_signer_key: Pubkey,
}

#[derive(Debug, Clone)]
pub struct Pool {
    pub amm: AmmKeys,
    pub initial_coin_balance: u64,
    pub initial_pc_balance: u64,
}
