use std::convert::TryInto;

use crate::{
    error::CryowarError,
    utils::{
        self, assert_pool_storage_account, assert_reward_vault, assert_signer,
        assert_staking_vault, assert_token_program, save_pool_storage_account,
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

pub fn process_fund_pool(
    accounts: &[AccountInfo],
    amount: u64,
    extend_duration: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let funder_wallet_account = next_account_info(account_info_iter)?;
    let cwar_pool_storage_account = next_account_info(account_info_iter)?;
    let staking_vault = next_account_info(account_info_iter)?;
    let cwar_rewards_vault = next_account_info(account_info_iter)?;
    let cwar_rewards_ata_to_debit = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    assert_signer(funder_wallet_account)?;
    assert_token_program(token_program)?;
    let mut cwar_pool_data_byte_array = cwar_pool_storage_account.data.try_borrow_mut().unwrap();
    let mut cwar_pool_data = assert_pool_storage_account(
        &cwar_pool_data_byte_array,
        cwar_pool_storage_account,
        program_id,
    )?;

    let mut is_funder_authorised = false;
    if *funder_wallet_account.key == cwar_pool_data.owner_wallet {
        is_funder_authorised = true;
    } else if cwar_pool_data
        .funders
        .iter()
        .any(|x| *x == *funder_wallet_account.key)
    {
        is_funder_authorised = true;
    }

    if !is_funder_authorised {
        msg!("CryowarError::FundingAuthorityMismatched");
        return Err(CryowarError::FundingAuthorityMismatched.into());
    }

    let cwar_staking_vault_data = TokenAccount::unpack(&staking_vault.data.borrow())?;
    let (pool_signer_address, _bump_seed) =
        Pubkey::find_program_address(&[&cwar_pool_storage_account.key.to_bytes()], program_id);
    assert_staking_vault(
        staking_vault,
        &cwar_pool_data,
        &cwar_staking_vault_data,
        &pool_signer_address,
    )?;

    let cwar_rewards_vault_data = TokenAccount::unpack(&cwar_rewards_vault.data.borrow())?;

    assert_reward_vault(
        cwar_rewards_vault,
        &cwar_pool_data,
        &cwar_rewards_vault_data,
        &pool_signer_address,
    )?;
    let now: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
    let total_cwar_staked = cwar_staking_vault_data.amount;
    utils::update_rewards(&mut cwar_pool_data, None, total_cwar_staked, now)?;

    msg!("now: {}", now);
    msg!(
        "reward_duration_end: {}",
        cwar_pool_data.reward_duration_end
    );
    msg!("amount: {}", amount);

    let mut updated_reward_amount = amount;

    //pool has not ended
    if now < cwar_pool_data.reward_duration_end {
        let remaining_duration = cwar_pool_data
            .reward_duration_end
            .checked_sub(now)
            .ok_or(CryowarError::AmountOverflow)?;
        let rewards_left_amount = remaining_duration
            .checked_mul(cwar_pool_data.reward_rate)
            .ok_or(CryowarError::AmountOverflow)?;
        updated_reward_amount = amount
            .checked_add(rewards_left_amount)
            .ok_or(CryowarError::AmountOverflow)?;
        cwar_pool_data.reward_duration_end = cwar_pool_data
            .reward_duration_end
            .checked_add(extend_duration)
            .ok_or(CryowarError::AmountOverflow)?;
    } else {
        cwar_pool_data.reward_duration_end = now
            .checked_add(extend_duration)
            .ok_or(CryowarError::AmountOverflow)?;
    }
    if cwar_pool_data.reward_duration_end <= now {
        msg!("CryowarError::DurationTooShort");
        return Err(CryowarError::DurationTooShort.into());
    }
    let new_reward_duration = cwar_pool_data
        .reward_duration_end
        .checked_sub(now)
        .ok_or(CryowarError::AmountOverflow)?;

    cwar_pool_data.reward_rate = updated_reward_amount
        .checked_div(new_reward_duration)
        .ok_or(CryowarError::AmountOverflow)?;
    cwar_pool_data.reward_duration = new_reward_duration;

    msg!("cwar_pool_data.reward_rate: {}", cwar_pool_data.reward_rate);
    cwar_pool_data.total_stake_last_update_time = now;

    if amount > 0 {
        msg!("Calling the token program to transfer CWAR rewards to Rewards Vault...");
        invoke(
            &spl_token::instruction::transfer(
                token_program.key,
                cwar_rewards_ata_to_debit.key,
                cwar_rewards_vault.key,
                funder_wallet_account.key,
                &[],
                amount,
            )?,
            &[
                cwar_rewards_ata_to_debit.clone(),
                cwar_rewards_vault.clone(),
                funder_wallet_account.clone(),
                token_program.clone(),
            ],
        )?;
    }

    save_pool_storage_account(&mut cwar_pool_data_byte_array, &cwar_pool_data)?;
    Ok(())
}
