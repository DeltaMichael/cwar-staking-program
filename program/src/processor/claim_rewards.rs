use std::convert::TryInto;

use crate::{
    error::CryowarError,
    utils::{
        self, assert_pool_storage_account, assert_reward_vault, assert_signer,
        assert_staking_vault, assert_token_account_to_be_owned_by_signer, assert_token_program,
        assert_user_storage_account, save_pool_storage_account, save_user_storage_account,
    },
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::state::Account as TokenAccount;

use super::unstake_cwar::check_locking_period;

pub fn process_claim_rewards(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let user_wallet_account = next_account_info(account_info_iter)?;
    let user_storage_account = next_account_info(account_info_iter)?;
    let cwar_pool_storage_account = next_account_info(account_info_iter)?;
    let staking_vault = next_account_info(account_info_iter)?;
    let cwar_rewards_vault = next_account_info(account_info_iter)?;
    let user_rewards_ata = next_account_info(account_info_iter)?;
    let pool_signer_pda = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    assert_signer(user_wallet_account)?;
    assert_token_account_to_be_owned_by_signer(user_rewards_ata, user_wallet_account)?;
    assert_token_program(token_program)?;
    let mut user_data_byte_array = user_storage_account.data.try_borrow_mut().unwrap();
    let mut user_storage_data = assert_user_storage_account(
        user_wallet_account,
        cwar_pool_storage_account,
        program_id,
        user_storage_account,
        &user_data_byte_array,
    )?;

    let mut cwar_pool_data_byte_array = cwar_pool_storage_account.data.try_borrow_mut().unwrap();
    let mut cwar_pool_data = assert_pool_storage_account(
        &cwar_pool_data_byte_array,
        cwar_pool_storage_account,
        program_id,
    )?;

    let (pool_signer_address, bump_seed) =
        Pubkey::find_program_address(&[&cwar_pool_storage_account.key.to_bytes()], program_id);
    if pool_signer_address != *pool_signer_pda.key {
        msg!("CryowarError::InvalidSignerPDA");
        return Err(CryowarError::InvalidSignerPDA.into());
    }

    let cwar_staking_vault_data = TokenAccount::unpack(&staking_vault.data.borrow())?;
    assert_staking_vault(
        staking_vault,
        &cwar_pool_data,
        &cwar_staking_vault_data,
        &pool_signer_address,
    )?;

    let now: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
    check_locking_period(&user_storage_data, &cwar_pool_data, now)?;

    let total_cwar_staked = cwar_staking_vault_data.amount;
    utils::update_rewards(
        &mut cwar_pool_data,
        Some(&mut user_storage_data),
        total_cwar_staked,
        now,
    )?;
    if user_storage_data.rewards_amount_pending > 0u64 {
        let mut reward_amount = user_storage_data.rewards_amount_pending;
        user_storage_data.rewards_amount_pending = 0u64;
        let cwar_rewards_vault_data = TokenAccount::unpack(&cwar_rewards_vault.data.borrow())?;
        assert_reward_vault(
            cwar_rewards_vault,
            &cwar_pool_data,
            &cwar_rewards_vault_data,
            &pool_signer_address,
        )?;
        let reward_vault_balance = cwar_rewards_vault_data.amount;
        if reward_vault_balance < reward_amount {
            reward_amount = reward_vault_balance;
        }

        if reward_amount > 0 {
            msg!("Calling the token program to transfer CWAR to User from Rewards Vault...");
            invoke_signed(
                &spl_token::instruction::transfer(
                    token_program.key,
                    cwar_rewards_vault.key,
                    user_rewards_ata.key,
                    &pool_signer_address,
                    &[&pool_signer_address],
                    reward_amount,
                )?,
                &[
                    cwar_rewards_vault.clone(),
                    user_rewards_ata.clone(),
                    pool_signer_pda.clone(),
                    token_program.clone(),
                ],
                &[&[&cwar_pool_storage_account.key.to_bytes(), &[bump_seed]]],
            )?;
        }

        let cwar_rewards_vault_data_after = TokenAccount::unpack(&cwar_rewards_vault.data.borrow())?;
        let actual_amount_withdrawn = cwar_rewards_vault_data
            .amount
            .checked_sub(cwar_rewards_vault_data_after.amount)
            .ok_or(CryowarError::AmountOverflow)?;

        if actual_amount_withdrawn != reward_amount {
            msg!("CryowarError::InvalidTransferOperation");
            return Err(CryowarError::InvalidTransferOperation.into());
        }
    }

    save_pool_storage_account(&mut cwar_pool_data_byte_array, &cwar_pool_data)?;
    save_user_storage_account(&mut user_data_byte_array, &user_storage_data)?;
    Ok(())
}
