use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum CryowarError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    /// Expected Amount Mismatch
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch,
    /// Amount Overflow
    #[error("Amount Overflow")]
    AmountOverflow,
    /// Account Not Owned By Cryowar Program
    #[error("Account Not Owned By Cryowar Program")]
    WrongAccountPassed,
    /// Some Other User Is Using This Space
    #[error("Space Not Empty")]
    SpaceNotEmpty,
    /// Expected account is not same as passed account
    #[error("Account Mismatched")]
    AccountMismatched,
    /// Expected Account Type Mismatched
    #[error("Expected Account Type Mismatched")]
    ExpectedAccountTypeMismatched,
    /// Invalid Token Program Id
    #[error("Invalid Token Program Id")]
    InvalidTokenProgram,
    /// Admin Does Not Matched
    #[error("Admin Does Not Matched")]
    AdminDoesNotMatched,
    ///PDA Account Does Not Matched
    #[error("PDA Account Does Not Matched")]
    PdaAccountDoesNotMatched,
    ///Data Size Not Matched
    #[error("Data Size Not Matched")]
    DataSizeNotMatched,
    ///Account Owner Should Be Token Program
    #[error("Account Owner Should Be Token Program")]
    AccountOwnerShouldBeTokenProgram,
    // Dervied Key Is Invalid
    #[error("Dervied Key Is Invalid")]
    DerivedKeyInvalid,
    ///User Storage Account Already Initialized
    #[error("User Storage Account Already Initialized")]
    UserStorageAccountAlreadyInitialized,
    /// Invalid System Program Id
    #[error("Invalid System Program Id")]
    InvalidSystemProgram,
    /// Duration Too Short
    #[error("Duration Too Short")]
    DurationTooShort,
    /// Mint Mismatched
    #[error("Mint Mismatched")]
    MintMismatched,
    /// User Storage Authority Mismatched
    #[error("User Storage Authority Mismatched")]
    UserStorageAuthorityMismatched,
    /// User Pool Mismatched
    #[error("User Pool Mismatched")]
    UserPoolMismatched,
    /// User Balance NonZero
    #[error("User Balance NonZero")]
    UserBalanceNonZero,
    // Invalid Staking Vault
    #[error("Invalid Staking Vault")]
    InvalidStakingVault,
    // Amount Must Be Greater Than Zero
    #[error("Amount Must Be Greater Than Zero")]
    AmountMustBeGreaterThanZero,
    // Insufficient Funds To Unstake
    #[error("Insufficient Funds To Unstake")]
    InsufficientFundsToUnstake,
    /// Pool Owner Mismatched
    #[error("Pool Owner Mismatched")]
    PoolOwnerMismatched,
    /// Pool Still Active
    #[error("Pool Still Active")]
    PoolStillActive,
    // Invalid Rewards Vault
    #[error("Invalid Rewards Vault")]
    InvalidRewardsVault,
    /// Funding Authority Mismatched
    #[error("Funding Authority Mismatched")]
    FundingAuthorityMismatched,
    /// Funder Already Present
    #[error("Funder Already Present")]
    FunderAlreadyPresent,
    /// Max Funders Reached
    #[error("Max Funders Reached")]
    MaxFundersReached,
    /// Cannot Remove Pool Owner
    #[error("Cannot Remove Pool Owner")]
    CannotRemovePoolOwner,
    /// Funder Is Not Present In Funder List
    #[error("Funder Is Not Present In Funder List")]
    FunderNotPresent,
    // Pool Adress Already Initialized
    #[error("Pool Adress Already Initialized")]
    PoolAddressAlreadyInitialized,
    // Provided seeds do not result in a valid address
    #[error("Provided seeds do not result in a valid address")]
    InvalidSeeds,
    ///Account Owner Should Be CWAR Program
    #[error("Account Owner Should Be CWAR Program")]
    AccountOwnerShouldBeCwarProgram,
    ///Pool Either Can Have Unstake Panelity Or Locking Duration
    #[error("Pool Either Can Have Unstake Panelity Or Locking Duration")]
    PoolEitherCanHaveUnstakePanelityOrLockingDuration,
    ///Locking Period Is Not Over Yet
    #[error("Locking Period Is Not Over Yet")]
    LockignPeriodIsNotOverYet,
    ///Invalid Authority Penality Deposit ATA
    #[error("Invalid Authority Penality Deposit ATA")]
    InvalidAuthorityPenalityDepositATA,
    ///Wrong Token Account Passed
    #[error("Wrong Token Account Passed")]
    WrongTokenAccountPassed,
    ///Invalid Signer PDA
    #[error("Invalid Signer PDA")]
    InvalidSignerPDA,
    ///Invalid Transfer Operation
    #[error("Invalid Transfer Operation")]
    InvalidTransferOperation,
    
}

impl From<CryowarError> for ProgramError {
    fn from(e: CryowarError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
