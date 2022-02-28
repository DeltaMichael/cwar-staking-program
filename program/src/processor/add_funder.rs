use crate::{
    error::CryowarError,
    utils::{assert_pool_storage_account, assert_signer, save_pool_storage_account},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

pub fn process_add_funder(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_owner_wallet_account = next_account_info(account_info_iter)?;
    let cwar_pool_storage_account = next_account_info(account_info_iter)?;
    let new_funder_wallet_account = next_account_info(account_info_iter)?;

    assert_signer(pool_owner_wallet_account)?;

    let mut cwar_pool_data_byte_array = cwar_pool_storage_account.data.try_borrow_mut().unwrap();

    let mut cwar_pool_data = assert_pool_storage_account(
        &cwar_pool_data_byte_array,
        cwar_pool_storage_account,
        program_id,
    )?;

    if cwar_pool_data.owner_wallet != *pool_owner_wallet_account.key {
        msg!("CryowarError::PoolOwnerMismatched");
        return Err(CryowarError::PoolOwnerMismatched.into());
    }

    if cwar_pool_data.owner_wallet == *new_funder_wallet_account.key {
        msg!("CryowarError::FunderAlreadyPresent");
        return Err(CryowarError::FunderAlreadyPresent.into());
    }
    if cwar_pool_data
        .funders
        .iter()
        .any(|x| *x == *new_funder_wallet_account.key)
    {
        msg!("CryowarError::FunderAlreadyPresent");
        return Err(CryowarError::FunderAlreadyPresent.into());
    }
    let default_pubkey = Pubkey::default();
    if let Some(idx) = cwar_pool_data
        .funders
        .iter()
        .position(|x| *x == default_pubkey)
    {
        cwar_pool_data.funders[idx] = *new_funder_wallet_account.key;
    } else {
        msg!("CryowarError::MaxFundersReached");
        return Err(CryowarError::MaxFundersReached.into());
    }

    save_pool_storage_account(&mut cwar_pool_data_byte_array, &cwar_pool_data)?;

    Ok(())
}
