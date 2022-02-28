use std::convert::TryInto;

use crate::{
    error::CryowarError,
    utils::{
        self, assert_pool_storage_account, assert_signer, assert_staking_vault,
        assert_token_program, assert_user_storage_account, save_pool_storage_account,
        save_user_storage_account,
    },
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::state::Account as TokenAccount;

pub fn process_stake_cwar(
    accounts: &[AccountInfo],
    amount_to_deposit: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let user_wallet_account = next_account_info(account_info_iter)?;
    let user_storage_account = next_account_info(account_info_iter)?;
    let cwar_pool_storage_account = next_account_info(account_info_iter)?;
    let staking_vault = next_account_info(account_info_iter)?;
    let user_cwar_ata = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    assert_signer(user_wallet_account)?;
    assert_token_program(token_program)?;
    let mut user_data_byte_array = user_storage_account.data.try_borrow_mut().unwrap();
    let mut user_storage_data = assert_user_storage_account(
        user_wallet_account,
        cwar_pool_storage_account,
        program_id,
        user_storage_account,
        &mut user_data_byte_array,
    )?;

    let mut cwar_pool_data_byte_array = cwar_pool_storage_account.data.try_borrow_mut().unwrap();
    let mut cwar_pool_data = assert_pool_storage_account(
        &cwar_pool_data_byte_array,
        cwar_pool_storage_account,
        program_id,
    )?;

    let (pool_signer_address, _bump_seed) =
        Pubkey::find_program_address(&[&cwar_pool_storage_account.key.to_bytes()], program_id);

    let cwar_staking_vault_data = TokenAccount::unpack(&staking_vault.data.borrow())?;
    assert_staking_vault(
        staking_vault,
        &cwar_pool_data,
        &cwar_staking_vault_data,
        &pool_signer_address,
    )?;

    if amount_to_deposit == 0u64 {
        msg!("CryowarError::AmountMustBeGreaterThanZero");
        return Err(CryowarError::AmountMustBeGreaterThanZero.into());
    }
    let now: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
    user_storage_data.unstake_penality_duration_end = cwar_pool_data.reward_duration_end;
    user_storage_data.last_staked_timestamp = now;

    let total_cwar_staked = cwar_staking_vault_data.amount;
    utils::update_rewards(
        &mut cwar_pool_data,
        Some(&mut user_storage_data),
        total_cwar_staked,
        now
    )?;

    msg!("Calling the token program to transfer CWAR to Staking Vault...");
    invoke(
        &spl_token::instruction::transfer(
            token_program.key,
            user_cwar_ata.key,
            staking_vault.key,
            user_wallet_account.key,
            &[],
            amount_to_deposit,
        )?,
        &[
            user_cwar_ata.clone(),
            staking_vault.clone(),
            user_wallet_account.clone(),
            token_program.clone(),
        ],
    )?;

    let cwar_staking_vault_data_after = TokenAccount::unpack(&staking_vault.data.borrow())?;
    let amount_deposited = cwar_staking_vault_data_after
        .amount
        .checked_sub(cwar_staking_vault_data.amount)
        .ok_or(CryowarError::AmountOverflow)?;
    user_storage_data.user_cwar_staked_amount = user_storage_data
        .user_cwar_staked_amount
        .checked_add(amount_deposited)
        .ok_or(CryowarError::AmountOverflow)?;
    save_pool_storage_account(&mut cwar_pool_data_byte_array, &cwar_pool_data)?;
    save_user_storage_account(&mut user_data_byte_array, &user_storage_data)?;

    Ok(())
}
