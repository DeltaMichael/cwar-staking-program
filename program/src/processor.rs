use crate::instruction::CryowarInstruction;

use {
    add_funder::process_add_funder, claim_rewards::process_claim_rewards,
    close_pool::process_close_pool, close_user::process_close_user,
    create_user::process_create_user, fund_pool::process_fund_pool,
    initialize_cwar_pool::process_initialize_cwar_pool, remove_funder::process_remove_funder,
    stake_cwar::process_stake_cwar, unstake_cwar::process_unstake_cwar,
};

pub mod add_funder;
pub mod claim_rewards;
pub mod close_pool;
pub mod close_user;
pub mod create_user;
pub mod fund_pool;
pub mod initialize_cwar_pool;
pub mod remove_funder;
pub mod stake_cwar;
pub mod unstake_cwar;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = CryowarInstruction::unpack(instruction_data)?;
        match instruction {
            CryowarInstruction::InitializeCwarPool {
                reward_duration,
                pool_nonce,
                unstake_penality_basis_points,
                locking_duration,
            } => {
                msg!("CryowarInstruction::InitializeCwarPool");
                process_initialize_cwar_pool(
                    accounts,
                    reward_duration,
                    pool_nonce,
                    unstake_penality_basis_points,
                    locking_duration,
                    program_id,
                )
            }
            CryowarInstruction::CreateUser { nonce } => {
                msg!("CryowarInstruction::CreateUser");
                process_create_user(accounts, nonce, program_id)
            }

            CryowarInstruction::StakeCwar { amount_to_deposit } => {
                msg!("CryowarInstruction::StakeCwar");
                process_stake_cwar(accounts, amount_to_deposit, program_id)
            }

            CryowarInstruction::UnstakeCwar { amount_to_withdraw } => {
                msg!("CryowarInstruction::UnstakeCwar");
                process_unstake_cwar(accounts, amount_to_withdraw, program_id)
            }

            CryowarInstruction::ClaimRewards {} => {
                msg!("CryowarInstruction::ClaimRewards");
                process_claim_rewards(accounts, program_id)
            }

            CryowarInstruction::AddFunder {} => {
                msg!("CryowarInstruction::AddFunder");
                process_add_funder(accounts, program_id)
            }

            CryowarInstruction::RemoveFunder {} => {
                msg!("CryowarInstruction::RemoveFunder");
                process_remove_funder(accounts, program_id)
            }

            CryowarInstruction::FundPool {
                amount,
                extend_duration,
            } => {
                msg!("CryowarInstruction::FundPool");
                process_fund_pool(accounts, amount, extend_duration, program_id)
            }

            CryowarInstruction::ClosePool {} => {
                msg!("CryowarInstruction::ClosePool");
                process_close_pool(accounts, program_id)
            }

            CryowarInstruction::CloseUser {} => {
                msg!("CryowarInstruction::CloseUser");
                process_close_user(accounts, program_id)
            }
        }
    }
}
