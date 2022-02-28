use std::cell::RefMut;
use std::convert::TryInto;

use crate::error::CryowarError;
use crate::processor::create_user::get_user_storage_address_and_bump_seed;
use crate::state::{
    AccTypesWithVersion, CwarPool, User, CWAR_POOL_STORAGE_TOTAL_BYTES, USER_STORAGE_TOTAL_BYTES,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::{msg, system_program};
// to avoid rounding errors
const PRECISION: u128 = u64::MAX as u128;
pub const FRACTION_TO_BASIS_POINTS: u128 = 10_000u128;
use spl_token::state::Account as TokenAccount;
pub mod constants {
    pub const CRYOWAR_TOKEN_MINT_PUBKEY: &str = "HfYFjMKNZygfMC8LsQ8LtpPsPxEJoXJx4M6tqi75Hajo";
    pub const MIN_DURATION: u64 = 86400;
}

pub fn close_account(
    account_to_close: &AccountInfo,
    sol_receiving_account: &AccountInfo,
    account_to_close_data_byte_array: &mut RefMut<&mut [u8]>,
) -> Result<(), CryowarError> {
    **sol_receiving_account.lamports.borrow_mut() = sol_receiving_account
        .lamports()
        .checked_add(account_to_close.lamports())
        .ok_or(CryowarError::AmountOverflow)?;
    **account_to_close.lamports.borrow_mut() = 0;
    **account_to_close_data_byte_array = &mut [];
    Ok(())
}

pub fn updated_rewards_per_token_accumulated(
    total_cwar_staked: u64,
    last_time_reward_applicable: u64,
    total_stake_last_update_time: u64,
    reward_rate: u64,
    rewards_per_token_accumulated_till_now: u128,
) -> Result<u128, ProgramError> {
    if total_cwar_staked == 0 {
        return Ok(rewards_per_token_accumulated_till_now);
    }
    let new_reward_per_token_stored: u128 = (reward_rate as u128)
        .checked_mul(
            (last_time_reward_applicable as u128)
                .checked_sub(total_stake_last_update_time as u128)
                .ok_or(CryowarError::AmountOverflow)?,
        )
        .ok_or(CryowarError::AmountOverflow)?;
    let new_reward_per_token_stored_with_precision: u128 = new_reward_per_token_stored
        .checked_mul(PRECISION)
        .ok_or(CryowarError::AmountOverflow)?;
    let updated_rewards_per_token_stored = rewards_per_token_accumulated_till_now
        .checked_add(
            new_reward_per_token_stored_with_precision
                .checked_div(total_cwar_staked as u128)
                .ok_or(CryowarError::AmountOverflow)?,
        )
        .ok_or(CryowarError::AmountOverflow)?;
    return Ok(updated_rewards_per_token_stored);
}

pub fn get_user_updated_pending_rewards(
    user_cwar_staked_amount: u64,
    rewards_per_token_accumulated_till_now: u128,
    rewards_per_token_accumulated_at_last_user_interaction: u128,
    rewards_amount_pending: u64,
) -> Result<u64, ProgramError> {
    let new_rewards_per_token_pending = rewards_per_token_accumulated_till_now
        .checked_sub(rewards_per_token_accumulated_at_last_user_interaction)
        .ok_or(CryowarError::AmountOverflow)?;
    let new_rewards_earned = assert_u128_to_u64_conversion(
        ((user_cwar_staked_amount as u128)
            .checked_mul(new_rewards_per_token_pending)
            .ok_or(CryowarError::AmountOverflow)?)
        .checked_div(PRECISION)
        .ok_or(CryowarError::AmountOverflow)?,
    )?;
    let updated_rewards_amount_pending = rewards_amount_pending
        .checked_add(new_rewards_earned)
        .ok_or(CryowarError::AmountOverflow)?;
    return Ok(updated_rewards_amount_pending);
}

pub fn update_rewards(
    cwar_pool: &mut CwarPool,
    user: Option<&mut User>,
    total_cwar_staked: u64,
    now: u64,
) -> ProgramResult {
    let last_time_reward_applicable =
        last_time_reward_applicable(cwar_pool.reward_duration_end, now);
    cwar_pool.rewards_per_token_accumulated_till_now = updated_rewards_per_token_accumulated(
        total_cwar_staked,
        last_time_reward_applicable,
        cwar_pool.total_stake_last_update_time,
        cwar_pool.reward_rate,
        cwar_pool.rewards_per_token_accumulated_till_now,
    )?;
    cwar_pool.total_stake_last_update_time = last_time_reward_applicable;

    if let Some(u) = user {
        u.rewards_amount_pending = get_user_updated_pending_rewards(
            u.user_cwar_staked_amount,
            cwar_pool.rewards_per_token_accumulated_till_now,
            u.rewards_per_token_accumulated_at_last_user_interaction,
            u.rewards_amount_pending,
        )?;
        u.rewards_per_token_accumulated_at_last_user_interaction =
            cwar_pool.rewards_per_token_accumulated_till_now;
    }

    Ok(())
}

pub fn assert_u128_to_u64_conversion(value: u128) -> Result<u64, ProgramError> {
    if value < u64::MAX as u128 {
        return Ok(value as u64);
    }
    msg!("CryowarError::AmountOverflow");
    return Err(CryowarError::AmountOverflow.into());
}
pub fn last_time_reward_applicable(reward_duration_end: u64, now_unix_timestamp: u64) -> u64 {
    return std::cmp::min(now_unix_timestamp.try_into().unwrap(), reward_duration_end);
}

pub fn assert_signer(signer_wallet_to_check: &AccountInfo) -> ProgramResult {
    if !signer_wallet_to_check.is_signer {
        msg!("ProgramError::MissingRequiredSignature");
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

pub fn assert_token_program(token_program_input: &AccountInfo) -> ProgramResult {
    if token_program_input.key != &spl_token::id() {
        msg!("CryowarError::InvalidTokenProgram");
        return Err(CryowarError::InvalidTokenProgram.into());
    }
    Ok(())
}

pub fn assert_system_program(system_program_input: &AccountInfo) -> ProgramResult {
    if system_program_input.key != &system_program::id() {
        msg!("CryowarError::InvalidSystemProgram");
        return Err(CryowarError::InvalidSystemProgram.into());
    }
    Ok(())
}

pub fn assert_staking_vault(
    staking_vault_input: &AccountInfo,
    cwar_pool_data: &CwarPool,
    staking_vault_data: &TokenAccount,
    pool_signer_address: &Pubkey,
) -> ProgramResult {
    if staking_vault_data.owner != *pool_signer_address {
        msg!("CryowarError::InvalidStakingVault");
        return Err(CryowarError::InvalidStakingVault.into());
    }

    if staking_vault_input.owner != &spl_token::id() {
        msg!("CryowarError::AccountOwnerShouldBeTokenProgram");
        return Err(CryowarError::AccountOwnerShouldBeTokenProgram.into());
    }
    if *staking_vault_input.key != cwar_pool_data.staking_vault {
        msg!("CryowarError::InvalidStakingVault");
        return Err(CryowarError::InvalidStakingVault.into());
    }
    Ok(())
}

pub fn assert_penality_deposit_ata(
    authority_penality_deposit_ata_account: &AccountInfo,
    cwar_pool_data: &CwarPool,
) -> ProgramResult {
    if cwar_pool_data.authority_penality_deposit_ata != *authority_penality_deposit_ata_account.key
    {
        msg!("CryowarError::InvalidAuthorityPenalityDepositATA");
        return Err(CryowarError::InvalidAuthorityPenalityDepositATA.into());
    }
    Ok(())
}

pub fn assert_reward_vault(
    reward_vault_input: &AccountInfo,
    cwar_pool_data: &CwarPool,
    rewards_vault_data: &TokenAccount,
    pool_signer_address: &Pubkey,
) -> ProgramResult {
    if rewards_vault_data.owner != *pool_signer_address {
        msg!("CryowarError::InvalidRewardsVault");
        return Err(CryowarError::InvalidRewardsVault.into());
    }
    if reward_vault_input.owner != &spl_token::id() {
        msg!("CryowarError::AccountOwnerShouldBeTokenProgram");
        return Err(CryowarError::AccountOwnerShouldBeTokenProgram.into());
    }
    if *reward_vault_input.key != cwar_pool_data.reward_vault {
        msg!("CryowarError::InvalidRewardsVault");
        return Err(CryowarError::InvalidRewardsVault.into());
    }
    Ok(())
}

pub fn assert_pool_storage_account(
    cwar_pool_data_byte_array: &[u8],
    cwar_pool_storage_account: &AccountInfo,
    program_id: &Pubkey,
) -> Result<CwarPool, CryowarError> {
    if cwar_pool_storage_account.owner != program_id {
        msg!("CryowarError::PoolAccountOwnerShouldBeCwarProgram");
        return Err(CryowarError::AccountOwnerShouldBeCwarProgram.into());
    }
    if cwar_pool_data_byte_array.len() != CWAR_POOL_STORAGE_TOTAL_BYTES {
        msg!("CryowarError::DataSizeNotMatched");
        return Err(CryowarError::DataSizeNotMatched.into());
    }
    let cwar_pool_data =
        CwarPool::try_from_slice(&cwar_pool_data_byte_array[0usize..CWAR_POOL_STORAGE_TOTAL_BYTES])
            .unwrap();
    if cwar_pool_data.acc_type != AccTypesWithVersion::CwarPoolDataV1 as u8 {
        msg!("CryowarError::ExpectedAccountTypeMismatched");
        return Err(CryowarError::ExpectedAccountTypeMismatched.into());
    }
    Ok(cwar_pool_data)
}

pub fn assert_user_storage_account(
    user_wallet_account: &AccountInfo,
    cwar_pool_storage_account: &AccountInfo,
    program_id: &Pubkey,
    user_storage_account: &AccountInfo,
    user_data_byte_array: &[u8],
) -> Result<User, CryowarError> {
    if user_storage_account.owner != program_id {
        msg!("CryowarError::UserAccountOwnerShouldBeCwarProgram");
        return Err(CryowarError::AccountOwnerShouldBeCwarProgram.into());
    }
    let (user_storage_address, _bump_seed) = get_user_storage_address_and_bump_seed(
        user_wallet_account.key,
        cwar_pool_storage_account.key,
        program_id,
    );
    if user_storage_address != *user_storage_account.key {
        msg!("Error: User Storage address does not match seed derivation");
        return Err(CryowarError::InvalidSeeds.into());
    }
    if user_data_byte_array.len() != USER_STORAGE_TOTAL_BYTES {
        msg!("CryowarError::DataSizeNotMatched");
        return Err(CryowarError::DataSizeNotMatched.into());
    }
    let user_storage_data: User =
        User::try_from_slice(&user_data_byte_array[0usize..USER_STORAGE_TOTAL_BYTES]).unwrap();
    if user_storage_data.acc_type != AccTypesWithVersion::UserDataV1 as u8 {
        msg!("CryowarError::ExpectedAccountTypeMismatched");
        return Err(CryowarError::ExpectedAccountTypeMismatched.into());
    }

    if user_storage_data.user_wallet != *user_wallet_account.key {
        msg!("CryowarError::UserStorageAuthorityMismatched");
        return Err(CryowarError::UserStorageAuthorityMismatched.into());
    }
    if user_storage_data.cwar_pool != *cwar_pool_storage_account.key {
        msg!("CryowarError::UserPoolMismatched");
        return Err(CryowarError::UserPoolMismatched.into());
    }
    Ok(user_storage_data)
}

pub fn save_user_storage_account(
    user_data_byte_array: &mut [u8],
    user_storage_data: &User,
) -> ProgramResult {
    user_data_byte_array[0usize..USER_STORAGE_TOTAL_BYTES]
        .copy_from_slice(&user_storage_data.try_to_vec().unwrap());
    Ok(())
}

pub fn save_pool_storage_account(
    cwar_pool_data_byte_array: &mut [u8],
    cwar_pool_data: &CwarPool,
) -> ProgramResult {
    cwar_pool_data_byte_array[0usize..CWAR_POOL_STORAGE_TOTAL_BYTES]
        .copy_from_slice(&cwar_pool_data.try_to_vec().unwrap());
    Ok(())
}

pub fn assert_token_account_to_be_owned_by_signer(
    token_account: &AccountInfo,
    signer_wallet: &AccountInfo,
) -> ProgramResult {
    let token_account_data = TokenAccount::unpack(&token_account.data.borrow())?;
    if token_account_data.owner != *signer_wallet.key {
        msg!("PhantasiaError::WrongTokenAccountPassed");
        return Err(CryowarError::WrongTokenAccountPassed.into());
    }
    Ok(())
}
