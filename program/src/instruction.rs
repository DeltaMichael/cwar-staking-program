use crate::error::CryowarError::InvalidInstruction;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;
pub enum CryowarInstruction {
    /// Accounts Expected:
    ///
    /// 0. `[signer]` Pool Owner Wallet Account
    /// 1. `[writable]` CWAR Pool Storage Account
    /// 2. `[]` CWAR Staking Mint
    /// 3. `[writable]` CWAR Staking Vault
    /// 4. `[]` CWAR Rewards Mint
    /// 5. `[writable]` CWAR Rewards Vault
    /// 6. `[]` Token Program
    /// 7. `[]` Authority Unstake Penality Deposit ATA
    InitializeCwarPool {
        reward_duration: u64,
        pool_nonce: u8,
        unstake_penality_basis_points: u16,
        locking_duration: u64,
    },

    /// 0. `[signer]` User Wallet Account
    /// 1. `[writable]` User Storage Account [user wallet, pool storage, program id]
    /// 2. `[writable]` CWAR Pool Storage Account
    /// 3. `[]` System Program
    CreateUser { nonce: u8 },

    /// 0. `[signer]` User Wallet Account
    /// 1. `[writable]` User Storage Account
    /// 2. `[writable]` CWAR Pool Storage Account
    /// 3. `[writable]` CWAR Staking Vault
    /// 4. `[writable]` CWAR ATA to Debit
    /// 5. `[]` Token Program
    StakeCwar { amount_to_deposit: u64 },

    /// 0. `[signer]` User Wallet Account
    /// 1. `[writable]` User Storage Account [user wallet, pool storage, program id] findProgramAddress
    /// 2. `[writable]` CWAR Pool Storage Account
    /// 3. `[writable]` CWAR Staking Vault
    /// 4. `[writable]` CWAR ATA to Credit
    /// 5. `[]` Pool Signer [pool storage, program id]
    /// 6. `[]` Token Program
    /// 7. `[writable]` Authority Unstake Penality Deposit ATA
    UnstakeCwar { amount_to_withdraw: u64 },

    /// 0. `[signer]` User Wallet Account
    /// 1. `[writable]` User Storage Account
    /// 2. `[writable]` CWAR Pool Storage Account
    /// 3. `[writable]` CWAR Staking Vault
    /// 4. `[writable]` CWAR Reward Vault
    /// 5. `[writable]` User Rewards ATA to Credit
    /// 6. `[]` Pool Signer [pool storage, program id] findProgramAddress
    /// 7. `[]` Token Program
    ClaimRewards {},

    /// 0. `[signer]` Pool Owner Wallet Account
    /// 1. `[writable]` CWAR Pool Storage Account
    /// 2. `[]` New Funder Wallet To Add
    AddFunder {},

    /// 0. `[signer]` Pool Owner Wallet Account
    /// 1. `[writable]` CWAR Pool Storage Account
    /// 2. `[]` Funder Wallet To Remove
    RemoveFunder {},

    /// 0. `[signer]` Funder Wallet Account
    /// 1. `[writable]` CWAR Pool Storage Account
    /// 2. `[writable]` CWAR Staking Vault
    /// 3. `[writable]` CWAR Reward Vault
    /// 4. `[writable]` CWAR ATA to Debit (Reward Token)
    /// 5. `[]` Token Program
    FundPool { amount: u64, extend_duration: u64 },

    /// 0. `[signer]` Pool Owner Wallet Account
    /// 1. `[writable]` CWAR Staking Vault
    /// 2. `[writable]` CWAR Staking Refund ATA
    /// 3. `[writable]` CWAR Rewards Vault
    /// 4. `[writable]` CWAR Rewards Refund ATA
    /// 5. `[writable]` CWAR Pool Storage Account
    /// 6. `[]` Pool Signer [pool storage, program id]
    /// 7. `[]` Token Program
    ClosePool {},

    /// 0. `[signer]` User Wallet Account
    /// 1. `[writable]` User Storage Account
    /// 2. `[writable]` CWAR Pool Storage Account
    CloseUser {},
}

impl CryowarInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        Ok(match input[0] {
            0 => Self::InitializeCwarPool {
                reward_duration: Self::unpack_to_u64(&input[1..9])?,
                pool_nonce: input[9],
                unstake_penality_basis_points: Self::unpack_to_u16(&input[10..12])?,
                locking_duration: Self::unpack_to_u64(&input[12..20])?,
            },
            1 => Self::CreateUser { nonce: input[1] },
            2 => Self::StakeCwar {
                amount_to_deposit: Self::unpack_to_u64(&input[1..9])?,
            },
            3 => Self::UnstakeCwar {
                amount_to_withdraw: Self::unpack_to_u64(&input[1..9])?,
            },

            4 => Self::ClaimRewards {},

            5 => Self::AddFunder {},

            6 => Self::RemoveFunder {},

            7 => Self::FundPool {
                amount: Self::unpack_to_u64(&input[1..9])?,
                extend_duration: Self::unpack_to_u64(&input[9..17])?,
            },

            8 => Self::ClosePool {},

            9 => Self::CloseUser {},

            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_to_u64(input: &[u8]) -> Result<u64, ProgramError> {
        let out_value = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(out_value)
    }

    fn unpack_to_u16(input: &[u8]) -> Result<u16, ProgramError> {
        let out_value = input
            .get(..2)
            .and_then(|slice| slice.try_into().ok())
            .map(u16::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(out_value)
    }
}
