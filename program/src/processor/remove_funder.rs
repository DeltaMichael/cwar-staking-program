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

pub fn process_remove_funder(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_owner_wallet_account = next_account_info(account_info_iter)?;
    let cwar_pool_storage_account = next_account_info(account_info_iter)?;
    let funder_wallet_to_remove = next_account_info(account_info_iter)?;

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

    if *funder_wallet_to_remove.key == cwar_pool_data.owner_wallet {
        msg!("CryowarError::CannotRemovePoolOwner");
        return Err(CryowarError::CannotRemovePoolOwner.into());
    }

    if let Some(idx) = cwar_pool_data
        .funders
        .iter()
        .position(|x| *x == *funder_wallet_to_remove.key)
    {
        cwar_pool_data.funders[idx] = Pubkey::default();
    } else {
        msg!("CryowarError::FunderNotPresent");
        return Err(CryowarError::FunderNotPresent.into());
    }

    save_pool_storage_account(&mut cwar_pool_data_byte_array, &cwar_pool_data)?;

    Ok(())
}
