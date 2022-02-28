use crate::{
    error::CryowarError,
    state,
    state::User,
    utils::{
        assert_pool_storage_account, assert_signer, assert_system_program,
        save_pool_storage_account, save_user_storage_account,
    },
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use std::convert::TryInto;

pub fn process_create_user(
    accounts: &[AccountInfo],
    nonce: u8,
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let user_wallet_account = next_account_info(account_info_iter)?;
    let user_storage_account = next_account_info(account_info_iter)?;
    let cwar_pool_storage_account = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    msg!("nonce: {}", nonce);

    assert_signer(user_wallet_account)?;

    let mut cwar_pool_data_byte_array = cwar_pool_storage_account.data.try_borrow_mut().unwrap();
    let mut cwar_pool_data = assert_pool_storage_account(
        &cwar_pool_data_byte_array,
        cwar_pool_storage_account,
        program_id,
    )?;

    assert_system_program(system_program_info)?;

    if !user_storage_account.data_is_empty() || user_storage_account.data_len() != 0 {
        msg!("CryowarError::UserStorageAccountAlreadyInitialized");
        return Err(CryowarError::UserStorageAccountAlreadyInitialized.into());
    }

    let (user_storage_address, bump_seed) = get_user_storage_address_and_bump_seed(
        user_wallet_account.key,
        cwar_pool_storage_account.key,
        program_id,
    );
    if user_storage_address != *user_storage_account.key {
        msg!("Error: User Storage address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    let user_storage_account_signer_seeds: &[&[_]] = &[
        &user_wallet_account.key.to_bytes(),
        &cwar_pool_storage_account.key.to_bytes(),
        &[bump_seed],
    ];

    create_and_allocate_account_raw(
        *program_id,
        user_storage_account,
        system_program_info,
        user_wallet_account,
        state::USER_STORAGE_TOTAL_BYTES,
        user_storage_account_signer_seeds,
    )
    .unwrap();

    let user_storage_data = User {
        acc_type: state::AccTypesWithVersion::UserDataV1 as u8,
        user_wallet: *user_wallet_account.key,
        cwar_pool: *cwar_pool_storage_account.key,
        user_cwar_staked_amount: 0u64,
        nonce: bump_seed,
        rewards_amount_pending: 0u64,
        rewards_per_token_accumulated_at_last_user_interaction: 0u128,
        unstake_penality_duration_end: cwar_pool_data.reward_duration_end,
        last_staked_timestamp: 0u64,
    };

    let mut user_data_byte_array = user_storage_account.data.try_borrow_mut().unwrap();

    cwar_pool_data.user_stake_count += 1u32;

    save_pool_storage_account(&mut cwar_pool_data_byte_array, &cwar_pool_data)?;
    save_user_storage_account(&mut user_data_byte_array, &user_storage_data)?;

    Ok(())
}

#[inline(always)]
pub fn create_and_allocate_account_raw<'a>(
    owner_program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let rent = Rent::get()?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);
        invoke(
            &system_instruction::transfer(payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    msg!("Allocate space for the account");
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        &[new_account_info.clone(), system_program_info.clone()],
        &[signer_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &owner_program_id),
        &[new_account_info.clone(), system_program_info.clone()],
        &[signer_seeds],
    )?;
    msg!("Completed assignation!");

    Ok(())
}

pub fn assert_derivation(
    program_id: &Pubkey,
    account: &AccountInfo,
    path: &[&[u8]],
) -> Result<u8, ProgramError> {
    let (key, bump) = Pubkey::find_program_address(&path, program_id);
    if key != *account.key {
        return Err(CryowarError::DerivedKeyInvalid.into());
    }
    Ok(bump)
}

/// Derives the user storage account address for the given wallet and pool
pub fn get_user_storage_address(
    user_wallet: &Pubkey,
    pool_storage: &Pubkey,
    program_id: &Pubkey,
) -> Pubkey {
    get_user_storage_address_and_bump_seed(user_wallet, pool_storage, program_id).0
}

pub fn get_user_storage_address_and_bump_seed(
    user_wallet: &Pubkey,
    pool_storage: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&user_wallet.to_bytes(), &pool_storage.to_bytes()],
        program_id,
    )
}
