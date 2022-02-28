use crate::{
    error::CryowarError,
    utils::{
        self, assert_pool_storage_account, assert_signer, assert_user_storage_account,
        save_pool_storage_account,
    },
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

pub fn process_close_user(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let user_wallet_account = next_account_info(account_info_iter)?;
    let user_storage_account = next_account_info(account_info_iter)?;
    let cwar_pool_storage_account = next_account_info(account_info_iter)?;

    assert_signer(user_wallet_account)?;

    let mut user_data_byte_array = user_storage_account.data.try_borrow_mut().unwrap();
    let user_storage_data = assert_user_storage_account(
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

    cwar_pool_data.user_stake_count -= 1u32;

    save_pool_storage_account(&mut cwar_pool_data_byte_array, &cwar_pool_data)?;

    if user_storage_data.user_cwar_staked_amount != 0u64
        || user_storage_data.rewards_amount_pending != 0
    {
        msg!("CryowarError::UserBalanceNonZero");
        return Err(CryowarError::UserBalanceNonZero.into());
    }

    msg!("Closing the User Data Storage account and transferring lamports to User wallet...");
    utils::close_account(
        user_storage_account,
        user_wallet_account,
        &mut user_data_byte_array,
    )
    .unwrap();
    Ok(())
}
