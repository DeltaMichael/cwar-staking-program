use crate::{
    error::CryowarError,
    utils::{
        assert_pool_storage_account, assert_reward_vault, assert_signer, assert_staking_vault,
        assert_token_program, save_pool_storage_account,
    },
};
use solana_program::sysvar::clock::Clock;
use solana_program::sysvar::Sysvar;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Account as TokenAccount;

pub fn process_close_pool(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_owner_wallet_account = next_account_info(account_info_iter)?;
    let staking_vault = next_account_info(account_info_iter)?;
    let cwar_staking_refund_ata = next_account_info(account_info_iter)?;
    let cwar_rewards_vault = next_account_info(account_info_iter)?;
    let cwar_rewards_refund_ata = next_account_info(account_info_iter)?;
    let cwar_pool_storage_account = next_account_info(account_info_iter)?;
    let pool_signer_pda = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    assert_signer(pool_owner_wallet_account)?;
    assert_token_program(token_program)?;

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

    let cwar_rewards_vault_data = TokenAccount::unpack(&cwar_rewards_vault.data.borrow())?;

    assert_reward_vault(
        cwar_rewards_vault,
        &cwar_pool_data,
        &cwar_rewards_vault_data,
        &pool_signer_address,
    )?;

    if cwar_pool_data.owner_wallet != *pool_owner_wallet_account.key {
        msg!("CryowarError::PoolOwnerMismatched");
        return Err(CryowarError::PoolOwnerMismatched.into());
    }

    let total_cwar_staked = cwar_staking_vault_data.amount;

    let now = Clock::get()?.unix_timestamp;
    if cwar_pool_data.reward_duration_end <= 0u64
        || cwar_pool_data.reward_duration_end >= (now as u64)
        || cwar_pool_data.user_stake_count != 0u32
        || total_cwar_staked != 0u64
    {
        msg!("CryowarError::PoolStillActive");
        return Err(CryowarError::PoolStillActive.into());
    }

    msg!("Calling the token program to transfer CWAR to Staking Refundee from Staking Vault...");
    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            staking_vault.key,
            cwar_staking_refund_ata.key,
            &pool_signer_address,
            &[&pool_signer_address],
            cwar_staking_vault_data.amount,
        )?,
        &[
            staking_vault.clone(),
            cwar_staking_refund_ata.clone(),
            pool_signer_pda.clone(),
            token_program.clone(),
        ],
        &[&[&cwar_pool_storage_account.key.to_bytes()[..], &[bump_seed]]],
    )?;

    msg!("Calling the token program to transfer CWAR to Rewards Refundee from Rewards Vault...");
    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            cwar_rewards_vault.key,
            cwar_rewards_refund_ata.key,
            &pool_signer_address,
            &[&pool_signer_address],
            cwar_rewards_vault_data.amount,
        )?,
        &[
            cwar_rewards_vault.clone(),
            cwar_rewards_refund_ata.clone(),
            pool_signer_pda.clone(),
            token_program.clone(),
        ],
        &[&[&cwar_pool_storage_account.key.to_bytes()[..], &[bump_seed]]],
    )?;

    msg!("Calling the token program to close CWAR Staking Vault...");
    invoke_signed(
        &spl_token::instruction::close_account(
            token_program.key,
            staking_vault.key,
            pool_owner_wallet_account.key,
            &pool_signer_address,
            &[&pool_signer_address],
        )?,
        &[
            staking_vault.clone(),
            pool_owner_wallet_account.clone(),
            pool_signer_pda.clone(),
            token_program.clone(),
        ],
        &[&[&cwar_pool_storage_account.key.to_bytes()[..], &[bump_seed]]],
    )?;

    msg!("Calling the token program to close CWAR Rewards Vault...");
    invoke_signed(
        &spl_token::instruction::close_account(
            token_program.key,
            cwar_rewards_vault.key,
            pool_owner_wallet_account.key,
            &pool_signer_address,
            &[&pool_signer_address],
        )?,
        &[
            cwar_rewards_vault.clone(),
            pool_owner_wallet_account.clone(),
            pool_signer_pda.clone(),
            token_program.clone(),
        ],
        &[&[&cwar_pool_storage_account.key.to_bytes()[..], &[bump_seed]]],
    )?;

    cwar_pool_data.staking_vault = Pubkey::default();
    cwar_pool_data.reward_vault = Pubkey::default();
    save_pool_storage_account(&mut cwar_pool_data_byte_array, &cwar_pool_data)?;

    Ok(())
}
