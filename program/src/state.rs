use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub enum AccTypesWithVersion {
    CwarPoolDataV1 = 2,
    UserDataV1 = 3,
}

pub const CWAR_POOL_STORAGE_TOTAL_BYTES: usize = 416;
#[derive(Clone, BorshDeserialize, BorshSerialize, Copy, Debug)]
pub struct CwarPool {
    pub acc_type: u8,
    pub owner_wallet: Pubkey,
    pub staking_vault: Pubkey,
    pub staking_mint: Pubkey,
    pub reward_vault: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_rate: u64,
    pub reward_duration: u64,
    pub total_stake_last_update_time: u64,
    pub rewards_per_token_accumulated_till_now: u128,
    pub user_stake_count: u32,
    pub pda_nonce: u8,
    pub funders: [Pubkey; 5],
    pub reward_duration_end: u64,
    pub unstake_penality_basis_points: u16,
    pub locking_duration: u64,
    pub authority_penality_deposit_ata: Pubkey,
}

pub const USER_STORAGE_TOTAL_BYTES: usize = 114;
#[derive(Clone, BorshDeserialize, BorshSerialize, Copy, Debug)]
pub struct User {
    pub acc_type: u8,
    pub user_wallet: Pubkey,
    pub cwar_pool: Pubkey,
    pub user_cwar_staked_amount: u64,
    pub nonce: u8,
    pub rewards_amount_pending: u64,
    pub rewards_per_token_accumulated_at_last_user_interaction: u128,
    pub unstake_penality_duration_end: u64,
    pub last_staked_timestamp: u64,
}
