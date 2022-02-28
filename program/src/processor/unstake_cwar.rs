use std::convert::TryInto;

use crate::{
    error::CryowarError,
    state::{CwarPool, User},
    utils::{
        self, assert_penality_deposit_ata, assert_pool_storage_account, assert_signer,
        assert_staking_vault, assert_token_program, assert_u128_to_u64_conversion,
        assert_user_storage_account, save_pool_storage_account, save_user_storage_account,
        FRACTION_TO_BASIS_POINTS,
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

pub fn process_unstake_cwar(
    accounts: &[AccountInfo],
    amount_to_withdraw: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let user_wallet_account = next_account_info(account_info_iter)?;
    let user_storage_account = next_account_info(account_info_iter)?;
    let cwar_pool_storage_account = next_account_info(account_info_iter)?;
    let staking_vault = next_account_info(account_info_iter)?;
    let user_cwar_ata = next_account_info(account_info_iter)?;
    let pool_signer_pda = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let authority_penality_deposit_ata_account = next_account_info(account_info_iter)?;

    assert_signer(user_wallet_account)?;
    assert_token_program(token_program)?;

    if amount_to_withdraw == 0u64 {
        msg!("CryowarError::AmountMustBeGreaterThanZero");
        return Err(CryowarError::AmountMustBeGreaterThanZero.into());
    }

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

    let cwar_staking_vault_data = TokenAccount::unpack(&staking_vault.data.borrow())?;
    let (pool_signer_address, bump_seed) =
        Pubkey::find_program_address(&[&cwar_pool_storage_account.key.to_bytes()], program_id);

    assert_staking_vault(
        staking_vault,
        &cwar_pool_data,
        &cwar_staking_vault_data,
        &pool_signer_address,
    )?;

    assert_penality_deposit_ata(authority_penality_deposit_ata_account, &cwar_pool_data)?;

    if user_storage_data.user_cwar_staked_amount < amount_to_withdraw {
        msg!("CryowarError::InsufficientFundsToUnstake");
        return Err(CryowarError::InsufficientFundsToUnstake.into());
    }

    let now: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
    let total_cwar_staked = cwar_staking_vault_data.amount;
    utils::update_rewards(
        &mut cwar_pool_data,
        Some(&mut user_storage_data),
        total_cwar_staked,
        now,
    )?;

    let mut amount_sent_to_user = amount_to_withdraw;

    if user_storage_data.unstake_penality_duration_end > now
        && cwar_pool_data.unstake_penality_basis_points > 0u16
    {
        msg!("current timestamp: {}", now);
        msg!("user_storage_data.unstake_penality_duration_end: {}", user_storage_data.unstake_penality_duration_end);
        let penality_amount = assert_u128_to_u64_conversion(
            (amount_to_withdraw as u128)
                .checked_mul(cwar_pool_data.unstake_penality_basis_points as u128)
                .ok_or(CryowarError::AmountOverflow)?
                .checked_div(FRACTION_TO_BASIS_POINTS)
                .ok_or(CryowarError::AmountOverflow)?,
        )?;
        amount_sent_to_user = amount_to_withdraw
            .checked_sub(penality_amount)
            .ok_or(CryowarError::AmountOverflow)?;

        msg!("Calling the token program to transfer CWAR to Unstake Penality ATA from Staking Vault...");
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                staking_vault.key,
                authority_penality_deposit_ata_account.key,
                &pool_signer_address,
                &[&pool_signer_address],
                penality_amount,
            )?,
            &[
                staking_vault.clone(),
                authority_penality_deposit_ata_account.clone(),
                pool_signer_pda.clone(),
                token_program.clone(),
            ],
            &[&[&cwar_pool_storage_account.key.to_bytes(), &[bump_seed]]],
        )?;
    }

    check_locking_period(&user_storage_data, &cwar_pool_data, now)?;

    let cwar_staking_vault_data_before = TokenAccount::unpack(&staking_vault.data.borrow())?;
    msg!("Calling the token program to transfer CWAR to User from Staking Vault...");
    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            staking_vault.key,
            user_cwar_ata.key,
            &pool_signer_address,
            &[&pool_signer_address],
            amount_sent_to_user,
        )?,
        &[
            staking_vault.clone(),
            user_cwar_ata.clone(),
            pool_signer_pda.clone(),
            token_program.clone(),
        ],
        &[&[&cwar_pool_storage_account.key.to_bytes(), &[bump_seed]]],
    )?;
    let cwar_staking_vault_data_after = TokenAccount::unpack(&staking_vault.data.borrow())?;
    let actual_amount_withdrawn = cwar_staking_vault_data_before
        .amount
        .checked_sub(cwar_staking_vault_data_after.amount)
        .ok_or(CryowarError::AmountOverflow)?;

    if actual_amount_withdrawn > amount_to_withdraw {
        msg!("CryowarError::InvalidTransferOperation");
        return Err(CryowarError::InvalidTransferOperation.into());
    }
    user_storage_data.user_cwar_staked_amount = user_storage_data
        .user_cwar_staked_amount
        .checked_sub(amount_to_withdraw)
        .ok_or(CryowarError::AmountOverflow)?;

    save_pool_storage_account(&mut cwar_pool_data_byte_array, &cwar_pool_data)?;
    save_user_storage_account(&mut user_data_byte_array, &user_storage_data)?;

    Ok(())
}

pub fn check_locking_period(
    user_storage_data: &User,
    cwar_pool_data: &CwarPool,
    now: u64,
) -> ProgramResult {
    let user_stake_duration = now
        .checked_sub(user_storage_data.last_staked_timestamp)
        .ok_or(CryowarError::AmountOverflow)?;
    if user_stake_duration < cwar_pool_data.locking_duration
        && now < cwar_pool_data.reward_duration_end
    {
        msg!("CryowarError::LockignPeriodIsNotOverYet");
        msg!(
            "now - user_storage_data.last_staked_timestamp: {}",
            user_stake_duration
        );
        msg!(
            "cwar_pool_data.locking_duration: {}",
            cwar_pool_data.locking_duration
        );
        msg!(
            "user_storage_data.last_staked_timestamp: {}",
            user_storage_data.last_staked_timestamp
        );
        return Err(CryowarError::LockignPeriodIsNotOverYet.into());
    }
    Ok(())
}
