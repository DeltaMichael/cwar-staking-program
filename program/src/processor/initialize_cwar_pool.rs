use crate::{
    error::CryowarError,
    state::{AccTypesWithVersion, CwarPool, CWAR_POOL_STORAGE_TOTAL_BYTES},
    utils::{assert_signer, assert_token_program, save_pool_storage_account},
};

use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use spl_token::state::Account as TokenAccount;

pub fn process_initialize_cwar_pool(
    accounts: &[AccountInfo],
    reward_duration: u64,
    pool_nonce: u8,
    unstake_penality_basis_points_input: u16,
    locking_duration_input: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_owner_wallet_account = next_account_info(account_info_iter)?;
    let cwar_pool_storage_account = next_account_info(account_info_iter)?;
    let staking_mint = next_account_info(account_info_iter)?;
    let staking_vault = next_account_info(account_info_iter)?;
    let cwar_rewards_mint = next_account_info(account_info_iter)?;
    let cwar_rewards_vault = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let authority_penality_deposit_ata_account = next_account_info(account_info_iter)?;

    msg!("pool_nonce: {}", pool_nonce);
    msg!(
        "Pool Authority: {}",
        pool_owner_wallet_account.key.to_string()
    );
    msg!(
        "Pool Storage: {}",
        cwar_pool_storage_account.key.to_string()
    );
    msg!("Staking Mint: {}", staking_mint.key.to_string());
    msg!("Staking Vault: {}", staking_vault.key.to_string());
    msg!("Rewards Mint: {}", cwar_rewards_mint.key.to_string());
    msg!("Rewards Vault: {}", cwar_rewards_vault.key.to_string());

    assert_signer(pool_owner_wallet_account)?;
    assert_token_program(token_program)?;

    let rent = Rent::get()?;

    if !rent.is_exempt(staking_vault.lamports(), staking_vault.data_len()) {
        msg!("CryowarError::StakingVaultNotRentExempt");
        return Err(CryowarError::NotRentExempt.into());
    }

    if !rent.is_exempt(cwar_rewards_vault.lamports(), cwar_rewards_vault.data_len()) {
        msg!("CryowarError::RewardsVaultNotRentExempt");
        return Err(CryowarError::NotRentExempt.into());
    }

    if !rent.is_exempt(
        cwar_pool_storage_account.lamports(),
        cwar_pool_storage_account.data_len(),
    ) {
        msg!("CryowarError::PoolStorageNotRentExempt");
        return Err(CryowarError::NotRentExempt.into());
    }

    if cwar_pool_storage_account.data_len() != CWAR_POOL_STORAGE_TOTAL_BYTES {
        msg!("CryowarError::PoolStorageDataSizeNotMatched");
        return Err(CryowarError::DataSizeNotMatched.into());
    }

    let (pool_signer_address, bump_seed) =
        Pubkey::find_program_address(&[&cwar_pool_storage_account.key.to_bytes()], program_id);
    //let pool_signer_address = Pubkey::create_program_address(&[&cwar_pool_storage_account.key.to_bytes(), &[pool_nonce]], program_id)?;
    msg!("Calling the token program to transfer Staking vault account ownership to Cryowar Pool program...");
    invoke(
        &spl_token::instruction::set_authority(
            token_program.key,
            staking_vault.key,
            Some(&pool_signer_address),
            spl_token::instruction::AuthorityType::AccountOwner,
            pool_owner_wallet_account.key,
            &[&pool_owner_wallet_account.key],
        )?,
        &[
            staking_vault.clone(),
            pool_owner_wallet_account.clone(),
            token_program.clone(),
        ],
    )?;

    msg!("Calling the token program to transfer Rewards vault account ownership to Cryowar Pool program...");
    invoke(
        &spl_token::instruction::set_authority(
            token_program.key,
            cwar_rewards_vault.key,
            Some(&pool_signer_address),
            spl_token::instruction::AuthorityType::AccountOwner,
            pool_owner_wallet_account.key,
            &[&pool_owner_wallet_account.key],
        )?,
        &[
            cwar_rewards_vault.clone(),
            pool_owner_wallet_account.clone(),
            token_program.clone(),
        ],
    )?;

    let cwar_staking_vault_data = TokenAccount::unpack(&staking_vault.data.borrow())?;
    if cwar_staking_vault_data.mint != *staking_mint.key {
        msg!("CryowarError::MintMismatched");
        return Err(CryowarError::MintMismatched.into());
    }
    if cwar_staking_vault_data.owner != pool_signer_address {
        msg!("CryowarError::InvalidStakingVault");
        return Err(CryowarError::InvalidStakingVault.into());
    }
    if staking_vault.owner != &spl_token::id() {
        msg!("CryowarError::AccountOwnerShouldBeTokenProgram");
        return Err(CryowarError::AccountOwnerShouldBeTokenProgram.into());
    }

    let cwar_rewards_vault_data = TokenAccount::unpack(&cwar_rewards_vault.data.borrow())?;
    if cwar_rewards_vault_data.mint != *cwar_rewards_mint.key {
        msg!("CryowarError::MintMismatched");
        return Err(CryowarError::MintMismatched.into());
    }
    if cwar_rewards_vault_data.owner != pool_signer_address {
        msg!("CryowarError::InvalidRewardsVault");
        return Err(CryowarError::InvalidRewardsVault.into());
    }
    if cwar_rewards_vault.owner != &spl_token::id() {
        msg!("CryowarError::AccountOwnerShouldBeTokenProgram");
        return Err(CryowarError::AccountOwnerShouldBeTokenProgram.into());
    }

    let authority_penality_deposit_ata_data =
        TokenAccount::unpack(&authority_penality_deposit_ata_account.data.borrow())?;
    if authority_penality_deposit_ata_data.mint != *staking_mint.key {
        msg!("CryowarError::MintMismatched");
        return Err(CryowarError::MintMismatched.into());
    }
    if authority_penality_deposit_ata_data.owner != *pool_owner_wallet_account.key {
        msg!("CryowarError::InvalidAuthorityPenalityDepositATA");
        return Err(CryowarError::InvalidAuthorityPenalityDepositATA.into());
    }
    if authority_penality_deposit_ata_account.owner != &spl_token::id() {
        msg!("CryowarError::AccountOwnerShouldBeTokenProgram");
        return Err(CryowarError::AccountOwnerShouldBeTokenProgram.into());
    }

    let mut cwar_pool_data_byte_array = cwar_pool_storage_account.data.try_borrow_mut().unwrap();
    let mut cwar_pool_data: CwarPool =
        CwarPool::try_from_slice(&cwar_pool_data_byte_array[0usize..CWAR_POOL_STORAGE_TOTAL_BYTES])
            .unwrap();

    if cwar_pool_data.acc_type != 0 {
        msg!("CryowarError::PoolAddressAlreadyInitialized");
        return Err(CryowarError::PoolAddressAlreadyInitialized.into());
    }
    if unstake_penality_basis_points_input != 0u16 && locking_duration_input != 0u64 {
        msg!("CryowarError::PoolEitherCanHaveUnstakePanelityOrLockingDuration");
        return Err(CryowarError::PoolEitherCanHaveUnstakePanelityOrLockingDuration.into());
    }

    cwar_pool_data.acc_type = AccTypesWithVersion::CwarPoolDataV1 as u8;
    cwar_pool_data.owner_wallet = *pool_owner_wallet_account.key;
    cwar_pool_data.staking_vault = *staking_vault.key;
    cwar_pool_data.staking_mint = *staking_mint.key;
    cwar_pool_data.reward_vault = *cwar_rewards_vault.key;
    cwar_pool_data.reward_mint = *cwar_rewards_mint.key;
    cwar_pool_data.reward_rate = 0u64;
    cwar_pool_data.reward_duration = reward_duration;
    cwar_pool_data.total_stake_last_update_time = 0u64;
    cwar_pool_data.rewards_per_token_accumulated_till_now = 0u128;
    cwar_pool_data.user_stake_count = 0u32;
    cwar_pool_data.pda_nonce = bump_seed;
    cwar_pool_data.reward_duration_end = 0u64;
    cwar_pool_data.unstake_penality_basis_points = unstake_penality_basis_points_input;
    cwar_pool_data.locking_duration = locking_duration_input;
    cwar_pool_data.authority_penality_deposit_ata = *authority_penality_deposit_ata_account.key;

    save_pool_storage_account(&mut cwar_pool_data_byte_array, &cwar_pool_data)?;

    Ok(())
}
