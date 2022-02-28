//#![cfg(feature = "test-bpf")]
//cargo test-bpf -- --nocapture

use arrayref::{array_ref, array_refs};
use borsh::BorshDeserialize;
use cwar_token_staking::{state::*, utils::update_rewards, *};
use rand::Rng;
use solana_program::{system_instruction, system_program};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    transport::TransportError,
};

use spl_associated_token_account;
use std::{
    cmp::{max, min},
    convert::TryInto,
    println,
    str::FromStr,
    thread::{self},
    time::{Duration, Instant},
};
pub const SPL_TOKEN_STATE_ACCOUNT_LEN: usize = 165;
pub const SPL_TOKEN_STATE_MINT_LEN: usize = 82;

use solana_client::rpc_client::RpcClient;
use solana_validator::test_validator::*;

pub const TO_RAW_TOKEN: u64 = 1000_000_000u64;

pub fn assert_approx_eq_ui(num1: u64, num2: f64) {
    println!("num1: {}, num2: {}", num1, num2);
    assert!((to_ui_amount(num1) - num2).abs() < 0.0001f64);
}

pub fn assert_approx_eq_raw(num1: u64, num2: u64, print_values: bool) {
    if print_values {
        println!("num1: {}, num2: {}", to_ui_amount(num1), to_ui_amount(num2));
    }
    // println!("diff: {}", to_ui_amount(max(num1, num2) - min(num1, num2)));
    assert!(max(num1, num2) - min(num1, num2) < 100u64);
}

pub fn to_raw_amount(amount: f64) -> u64 {
    (amount * (TO_RAW_TOKEN as f64)) as u64
}

pub fn to_ui_amount(amount: u64) -> f64 {
    amount as f64 / TO_RAW_TOKEN as f64
}

pub struct ManualRandomTestPool {
    pub total_cwar_staked: u64,
    pub users_manual_pending_rewards: Vec<u64>,
    pub users_manual_completed_rewards: Vec<u64>,
    pub pool_data: CwarPool,
    pub users_data: Vec<User>,
    pub user_last_stake_timestamp: Vec<u64>,
}

impl ManualRandomTestPool {
    pub fn new(
        num_users: usize,
        reward_rate: u64,
        mut current_time: u64,
        reward_duration: u64,
    ) -> Self {
        // initialize and fund pool at t = 2, with pool duration as 86400 seconds and 86400 reward tokens

        let mut pool_data = CwarPool {
            acc_type: 2,
            owner_wallet: Pubkey::from_str("7LCUcgvVwzBbEtn3XXFQPUz8Gy9DVrDLR7TVcC7tvTqx").unwrap(),
            staking_vault: Pubkey::from_str("dmmTy2XWpXKWLaj5PGurAakk6Uq2YucQ5SRwZpQQyRy").unwrap(),
            staking_mint: Pubkey::from_str("9qXxzBFvN9kkuzFPMV6CExXfxRX9x9VFzGp2H4y7gryc").unwrap(),
            reward_vault: Pubkey::from_str("HFqTfAyHFEokqXeX2AEDnPvCZfe9QJr8iKAjKDoUYxfw").unwrap(),
            reward_mint: Pubkey::from_str("kQiVYvwJgk8tCgWDLTexfG1wT2aLMHedYBiFDFKX2or").unwrap(),
            reward_rate: 0,
            reward_duration,
            total_stake_last_update_time: 0,
            rewards_per_token_accumulated_till_now: 0,
            user_stake_count: 0,
            pda_nonce: 255,
            funders: [
                Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                Pubkey::from_str("11111111111111111111111111111111").unwrap(),
                Pubkey::from_str("11111111111111111111111111111111").unwrap(),
            ],
            reward_duration_end: 0,
            unstake_penality_basis_points: 0,
            locking_duration: 0,
            authority_penality_deposit_ata: Pubkey::from_str(
                "iAV39sXyuCBYqmHRg7zEZ4Y1V6qKFwCm52fTAyuucsU",
            )
            .unwrap(),
        };

        update_rewards(&mut pool_data, None, 0, current_time).unwrap();

        // fund the pool at t = 3
        pool_data.reward_rate = reward_rate;
        pool_data.total_stake_last_update_time = current_time;
        pool_data.reward_duration_end = reward_duration + current_time;

        let mut users_data: Vec<User> = Vec::with_capacity(num_users);

        // create num_users at t = 4
        pool_data.user_stake_count = num_users as u32;
        current_time += 1;
        for _i in 1..=num_users {
            users_data.push(User {
                acc_type: 3,
                user_wallet: Pubkey::from_str("GgC7ApXioMZBiqNmPDPqEtXfRHB6Zkh9pNwy6MEddwzR")
                    .unwrap(),
                cwar_pool: Pubkey::from_str("Hbzudz9AK5oWtBD1t1DockzxtQRfezynCie3b41mXGc9")
                    .unwrap(),
                user_cwar_staked_amount: 0,
                nonce: 255,
                rewards_amount_pending: 0,
                rewards_per_token_accumulated_at_last_user_interaction: 0,
                unstake_penality_duration_end: reward_duration + current_time,
                last_staked_timestamp: 0,
            })
        }
        Self {
            total_cwar_staked: 0u64,
            users_manual_pending_rewards: vec![0u64; num_users],
            users_manual_completed_rewards: vec![0u64; num_users],
            pool_data,
            users_data,
            user_last_stake_timestamp: vec![0u64; num_users],
        }
    }

    pub fn update_onchain_rewards(&mut self, user_index: usize, current_timestamp: u64) {
        update_rewards(
            &mut self.pool_data,
            Some(&mut self.users_data[user_index]),
            self.total_cwar_staked,
            current_timestamp,
        )
        .unwrap();
    }

    //stake
    // user_storage_data.unstake_penality_duration_end = cwar_pool_data.reward_duration_end;
    // user_storage_data.last_staked_timestamp = now as u64;
    //update cwar rewards
    //update total_cwar_staked
    //update user_cwar_staked_amount
    pub fn stake_cwar(&mut self, user_index: usize, amount_to_stake: u64, current_timestamp: u64) {
        self.users_data[user_index].unstake_penality_duration_end =
            self.pool_data.reward_duration_end;
        self.users_data[user_index].last_staked_timestamp = current_timestamp;
        self.update_onchain_rewards(user_index, current_timestamp);
        self.total_cwar_staked += amount_to_stake;
        self.users_data[user_index].user_cwar_staked_amount += amount_to_stake;
    }

    //unstake
    //update rewards
    //update total_cwar_staked
    // update user_cwar_staked_amount

    pub fn unstake_cwar(
        &mut self,
        user_index: usize,
        amount_to_unstake: u64,
        current_timestamp: u64,
    ) {
        self.update_onchain_rewards(user_index, current_timestamp);
        self.total_cwar_staked -= amount_to_unstake;
        self.users_data[user_index].user_cwar_staked_amount -= amount_to_unstake;
    }

    //claim rewards
    //update rewards
    //transfer user_storage_data.rewards_amount_pending to user
    pub fn claim_rewards(&mut self, user_index: usize, current_timestamp: u64) -> u64 {
        self.update_onchain_rewards(user_index, current_timestamp);
        let reward_amount = self.users_data[user_index].rewards_amount_pending;
        self.users_data[user_index].rewards_amount_pending = 0u64;
        self.users_manual_completed_rewards[user_index] +=
            self.users_manual_pending_rewards[user_index];
        self.users_manual_pending_rewards[user_index] = 0u64;
        reward_amount
    }

    pub fn get_user_rewards(&mut self, user_index: usize, current_timestamp: u64) -> u64 {
        self.update_onchain_rewards(user_index, current_timestamp);
        self.users_data[user_index].rewards_amount_pending
    }

    pub fn get_user_reward_for_this_second(
        &mut self,
        user_index: usize,
        current_timestamp: u64,
    ) -> u64 {
        if self.total_cwar_staked == 0 || current_timestamp > self.pool_data.reward_duration_end {
            return 0u64;
        }
        if self.user_last_stake_timestamp[user_index] == current_timestamp {
            return 0;
        }
        self.user_last_stake_timestamp[user_index] = current_timestamp;
        return ((self.pool_data.reward_rate as u128
            * self.users_data[user_index].user_cwar_staked_amount as u128)
            / (self.total_cwar_staked as u128)) as u64;
    }
    pub fn update_users_rewards_manually(&mut self, current_timestamp: u64) {
        for i in 0..self.users_data.len() {
            self.users_manual_pending_rewards[i] +=
                self.get_user_reward_for_this_second(i, current_timestamp);
        }
    }

    pub fn assert_manual_and_onchain_rewards(
        &mut self,
        current_timestamp: u64,
        print_values: bool,
    ) {
        for i in 0..self.users_data.len() {
            self.update_onchain_rewards(i, current_timestamp);
            self.users_manual_pending_rewards[i] +=
                self.get_user_reward_for_this_second(i, current_timestamp);
            assert_approx_eq_raw(
                self.users_data[i].rewards_amount_pending,
                self.users_manual_pending_rewards[i],
                print_values,
            );
        }
    }

    pub fn get_rewards_distributed_manually(&self) -> u64 {
        let mut total_rewards_distributed = 0u64;
        for i in 0..self.users_data.len() {
            total_rewards_distributed += self.users_manual_pending_rewards[i];
        }
        return total_rewards_distributed;
    }

    pub fn random_unstake_helper(&mut self, current_timestamp: u64) {
        let do_unstake = true;
        let mut rng = rand::thread_rng();
        let user_index = rng.gen_range(0..self.users_data.len());
        let amount_to_unstake = self.users_data[user_index].user_cwar_staked_amount;
        if amount_to_unstake > 0u64 && do_unstake {
            self.users_manual_pending_rewards[user_index] +=
                self.get_user_reward_for_this_second(user_index, current_timestamp);
            self.unstake_cwar(
                user_index,
                rng.gen_range(0..amount_to_unstake),
                current_timestamp,
            );
        }
    }

    pub fn random_stake_helper(&mut self, current_timestamp: u64) {
        let mut rng = rand::thread_rng();
        let user_index = rng.gen_range(0..self.users_data.len());
        let amount_to_stake = to_raw_amount(rng.gen_range(0.0..1000.0));

        self.users_manual_pending_rewards[user_index] +=
            self.get_user_reward_for_this_second(user_index, current_timestamp);

        self.stake_cwar(user_index, amount_to_stake, current_timestamp);
    }

    pub fn stake_and_unstake_randomly(&mut self, current_timestamp: u64) {
        let mut rng = rand::thread_rng();
        let operation_type: i32 = rng.gen_range(1..50);
        match operation_type {
            1 => {
                self.random_stake_helper(current_timestamp);
            }
            2 => {
                self.random_unstake_helper(current_timestamp);
            }
            3 => {
                self.random_stake_helper(current_timestamp);

                self.random_unstake_helper(current_timestamp);
            }
            4 => {
                self.random_unstake_helper(current_timestamp);
                self.random_stake_helper(current_timestamp);
            }
            5 => {
                self.random_stake_helper(current_timestamp);

                self.random_unstake_helper(current_timestamp);
                self.random_stake_helper(current_timestamp);
            }
            6 => {
                self.random_stake_helper(current_timestamp);

                self.random_stake_helper(current_timestamp);

                self.random_unstake_helper(current_timestamp);
            }
            7 => {
                self.random_unstake_helper(current_timestamp);
                self.random_unstake_helper(current_timestamp);
                self.random_stake_helper(current_timestamp);
            }
            8 => {
                self.random_unstake_helper(current_timestamp);
                self.random_stake_helper(current_timestamp);

                self.random_unstake_helper(current_timestamp);
            }
            _ => {}
        }
    }
}

// have 10 users and run for loop for 90000 seconds, users randomly stake, unstake, claim rewards
#[test]
fn test_claim_rewards_random() {
    let current_timestamp: u64;

    current_timestamp = 3;
    let mut test_pool =
        ManualRandomTestPool::new(10, to_raw_amount(1.0), current_timestamp, 86400 * 365);

    let run_upto_timestamp = 86400 * 400;
    for current_timestamp in 5u64..run_upto_timestamp {
        if current_timestamp % 1000 == 0 {
            test_pool.assert_manual_and_onchain_rewards(current_timestamp, true);
            println!("timestamp done: {}", current_timestamp);
            println!(
                "rewards distribured: {}",
                to_ui_amount(
                    test_pool.pool_data.reward_rate
                        * min(
                            current_timestamp - 5,
                            test_pool.pool_data.reward_duration_end - 5
                        )
                )
            );
            println!(
                "manual rewards: {}",
                to_ui_amount(test_pool.get_rewards_distributed_manually())
            );
        }
        test_pool.assert_manual_and_onchain_rewards(current_timestamp, false);
        test_pool.stake_and_unstake_randomly(current_timestamp);
    }
}

pub fn program_test(program_id: Pubkey) -> (TestValidator, Keypair) {
    let (test_validator, payer) = TestValidatorGenesis::default()
        .add_program("cwar_token_staking", program_id)
        .start();
    (test_validator, payer)
}

pub fn create_mint(
    rpc_client: &mut RpcClient,
    payer: &Keypair,
    mint_account: &Keypair,
    owner: &Pubkey,
) -> Result<(), TransportError> {
    let mint_rent = rpc_client
        .get_minimum_balance_for_rent_exemption(SPL_TOKEN_STATE_MINT_LEN)
        .unwrap();
    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint_account.pubkey(),
                mint_rent,
                SPL_TOKEN_STATE_MINT_LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint_account.pubkey(),
                &owner,
                None,
                9,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    transaction.sign(&[payer, mint_account], recent_blockhash);
    rpc_client
        .send_and_confirm_transaction(&transaction)
        .map_err(|err| format!("error: send transaction: {}", err))
        .unwrap();
    Ok(())
}

pub fn create_token_account(
    rpc_client: &mut RpcClient,
    payer: &Keypair,
    account: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Result<(), TransportError> {
    let account_rent = rpc_client
        .get_minimum_balance_for_rent_exemption(SPL_TOKEN_STATE_ACCOUNT_LEN)
        .unwrap();
    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &account.pubkey(),
                account_rent,
                SPL_TOKEN_STATE_ACCOUNT_LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                owner,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    transaction.sign(&[payer, account], recent_blockhash);
    rpc_client
        .send_and_confirm_transaction(&transaction)
        .map_err(|err| format!("error: send transaction: {}", err))
        .unwrap();
    Ok(())
}

pub fn create_data_account(
    rpc_client: &mut RpcClient,
    payer: &Keypair,
    account: &Keypair,
    space: u64,
    owner: &Pubkey,
) -> Result<(), TransportError> {
    let rent = rpc_client
        .get_minimum_balance_for_rent_exemption(space.try_into().unwrap())
        .unwrap();
    let mut transaction = Transaction::new_with_payer(
        &[system_instruction::create_account(
            &payer.pubkey(),
            &account.pubkey(),
            rent,
            space,
            owner,
        )],
        Some(&payer.pubkey()),
    );
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    transaction.sign(&[payer, account], recent_blockhash);

    rpc_client
        .send_and_confirm_transaction(&transaction)
        .map_err(|err| format!("error: send transaction: {}", err))
        .unwrap();
    Ok(())
}

pub fn create_ata(
    rpc_client: &mut RpcClient,
    payer: &Keypair,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<(), TransportError> {
    let mut transaction = Transaction::new_with_payer(
        &[
            spl_associated_token_account::create_associated_token_account(
                &payer.pubkey(),
                owner,
                mint,
            ),
        ],
        Some(&payer.pubkey()),
    );
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    transaction.sign(&[payer], recent_blockhash);
    rpc_client
        .send_and_confirm_transaction(&transaction)
        .map_err(|err| format!("error: send transaction: {}", err))
        .unwrap();
    Ok(())
}

pub fn airdrop_1_sol(
    rpc_client: &mut RpcClient,
    payer: &Keypair,
    receipient_wallet: &Pubkey,
) -> Result<(), TransportError> {
    let mut transaction = Transaction::new_with_payer(
        &[system_instruction::transfer(
            &payer.pubkey(),
            receipient_wallet,
            TO_RAW_TOKEN,
        )],
        Some(&payer.pubkey()),
    );
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    transaction.sign(&[payer], recent_blockhash);
    rpc_client
        .send_and_confirm_transaction(&transaction)
        .map_err(|err| format!("error: send transaction: {}", err))
        .unwrap();
    Ok(())
}

pub fn mint_tokens_to(
    rpc_client: &mut RpcClient,
    payer: &Keypair,
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Keypair,
    amount: u64,
) -> Result<(), TransportError> {
    let mut transaction = Transaction::new_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            destination,
            &authority.pubkey(),
            &[&authority.pubkey()],
            amount,
        )
        .unwrap()],
        Some(&payer.pubkey()),
    );
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    transaction.sign(&[payer, authority], recent_blockhash);
    rpc_client
        .send_and_confirm_transaction(&transaction)
        .map_err(|err| format!("error: send transaction: {}", err))
        .unwrap();
    Ok(())
}

pub fn get_token_balance(rpc_client: &mut RpcClient, token: &Pubkey) -> u64 {
    let token_account = rpc_client.get_account_data(token).unwrap();
    let src = array_ref![token_account, 0, 165];
    let (_mint, _owner, amount, _delegate, _state, _is_native, _delegated_amount, _close_authority) =
        array_refs![src, 32, 32, 8, 36, 1, 12, 8, 36];
    let amount = u64::from_le_bytes(*amount);
    return amount;
}

pub struct TestPool {
    pub program_id: Pubkey,
    pub test_validator: TestValidator,
    pub payer: Keypair,
    pub pool_info_account: Keypair,
    pub staking_vault_account: Keypair,
    pub rewards_vault_account: Keypair,
    pub staking_mint_account: Keypair,
    pub rewards_mint_account: Keypair,
    pub owner_account: Keypair,
    pub pool_signer_addr: Pubkey,
    pub reward_duration: u64,
    pub pool_nonce: u8,
    pub unstake_penality_basis_points: u16,
    pub locking_duration: u64,
    pub authority_unstake_penality_deposit_ata: Pubkey,
    pub user_wallet1: Keypair,
    pub user_wallet2: Keypair,
    pub user_wallet3: Keypair,
}

impl TestPool {
    pub fn new() -> Self {
        let program_id_keyp = Keypair::new();
        let (test_validator, payer) = program_test(program_id_keyp.pubkey());
        let pool_info_account = Keypair::new();
        let (pool_signer_address, bump_seed) = Pubkey::find_program_address(
            &[&pool_info_account.pubkey().to_bytes()],
            &program_id_keyp.pubkey(),
        );
        let staking_mint_account = Keypair::new();
        let owner_wallet = Keypair::new();
        let authority_unstake_penality_deposit_ata_calculated =
            spl_associated_token_account::get_associated_token_address(
                &owner_wallet.pubkey(),
                &staking_mint_account.pubkey(),
            );

        Self {
            program_id: program_id_keyp.pubkey(),
            test_validator,
            payer,
            pool_info_account,
            staking_vault_account: Keypair::new(),
            rewards_vault_account: Keypair::new(),
            staking_mint_account: staking_mint_account,
            rewards_mint_account: Keypair::new(),
            owner_account: owner_wallet,
            pool_signer_addr: pool_signer_address,
            reward_duration: 86400u64,
            pool_nonce: bump_seed,
            unstake_penality_basis_points: 0u16,
            locking_duration: 0u64,
            authority_unstake_penality_deposit_ata:
                authority_unstake_penality_deposit_ata_calculated,
            user_wallet1: Keypair::new(),
            user_wallet2: Keypair::new(),
            user_wallet3: Keypair::new(),
        }
    }

    pub fn get_user_storage_address(&self, user_wallet: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                &user_wallet.to_bytes(),
                &self.pool_info_account.pubkey().to_bytes(),
            ],
            &self.program_id,
        )
    }

    pub fn init_pool(&self, rpc_client: &mut RpcClient) {
        // create pool account
        create_data_account(
            rpc_client,
            &self.payer,
            &self.pool_info_account,
            state::CWAR_POOL_STORAGE_TOTAL_BYTES as u64,
            &self.program_id,
        )
        .unwrap();

        // create staking mint
        create_mint(
            rpc_client,
            &self.payer,
            &self.staking_mint_account,
            &self.owner_account.pubkey(),
        )
        .unwrap();

        //create rewards mint
        create_mint(
            rpc_client,
            &self.payer,
            &self.rewards_mint_account,
            &self.owner_account.pubkey(),
        )
        .unwrap();

        //create staking vault
        create_token_account(
            rpc_client,
            &self.payer,
            &self.staking_vault_account,
            &self.staking_mint_account.pubkey(),
            &self.owner_account.pubkey(),
        )
        .unwrap();

        //create rewards vault
        create_token_account(
            rpc_client,
            &self.payer,
            &self.rewards_vault_account,
            &self.rewards_mint_account.pubkey(),
            &self.owner_account.pubkey(),
        )
        .unwrap();

        //create Authority Unstake Penality Deposit ATA
        create_ata(
            rpc_client,
            &self.payer,
            &self.owner_account.pubkey(),
            &self.staking_mint_account.pubkey(),
        )
        .unwrap();

        //initialize pool
        let mut data = vec![0u8];
        data.extend(self.reward_duration.to_le_bytes());
        data.extend(self.pool_nonce.to_le_bytes());
        data.extend(self.unstake_penality_basis_points.to_le_bytes());
        data.extend(self.locking_duration.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(self.owner_account.pubkey(), true),
            AccountMeta::new(self.pool_info_account.pubkey(), false),
            AccountMeta::new_readonly(self.staking_mint_account.pubkey(), false),
            AccountMeta::new(self.staking_vault_account.pubkey(), false),
            AccountMeta::new_readonly(self.rewards_mint_account.pubkey(), false),
            AccountMeta::new(self.rewards_vault_account.pubkey(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new(self.authority_unstake_penality_deposit_ata, false),
        ];

        let mut transaction = Transaction::new_with_payer(
            &[Instruction {
                program_id: self.program_id,
                accounts,
                data,
            }],
            Some(&self.payer.pubkey()),
        );
        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
        transaction.sign(&[&self.payer, &self.owner_account], recent_blockhash);
        rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|err| format!("error: send transaction: {}", err))
            .unwrap();
    }

    pub fn fund_pool(&self, rpc_client: &mut RpcClient) {
        //fund pool with reward token
        let funder_reward_ata_calculated =
            spl_associated_token_account::get_associated_token_address(
                &self.owner_account.pubkey(),
                &self.rewards_mint_account.pubkey(),
            );

        //create funder ata of rewards mint
        create_ata(
            rpc_client,
            &self.payer,
            &self.owner_account.pubkey(),
            &self.rewards_mint_account.pubkey(),
        )
        .unwrap();

        //mint 86400 tokens to funder ata of rewards mint
        mint_tokens_to(
            rpc_client,
            &self.payer,
            &self.rewards_mint_account.pubkey(),
            &funder_reward_ata_calculated,
            &self.owner_account,
            86400u64 * TO_RAW_TOKEN,
        )
        .unwrap();

        //fund pool with 86400 rewards token and 1 day (86400 seconds)
        let mut data = vec![7u8];
        data.extend((86400u64 * TO_RAW_TOKEN).to_le_bytes());
        data.extend(86400u64.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(self.owner_account.pubkey(), true),
            AccountMeta::new(self.pool_info_account.pubkey(), false),
            AccountMeta::new(self.staking_vault_account.pubkey(), false),
            AccountMeta::new(self.rewards_vault_account.pubkey(), false),
            AccountMeta::new(funder_reward_ata_calculated, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];

        let mut transaction = Transaction::new_with_payer(
            &[Instruction {
                program_id: self.program_id,
                accounts,
                data,
            }],
            Some(&self.payer.pubkey()),
        );
        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
        transaction.sign(&[&self.payer, &self.owner_account], recent_blockhash);
        rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|err| format!("error: send transaction: {}", err))
            .unwrap();
    }

    pub fn create_user(&self, rpc_client: &mut RpcClient, user_wallet: &Keypair) {
        let (user_storage_address, bump_seed) =
            self.get_user_storage_address(&user_wallet.pubkey());

        let user_staking_ata = self.get_user_staking_ata(&user_wallet.pubkey());

        //create user rewards ata
        create_ata(
            rpc_client,
            &self.payer,
            &user_wallet.pubkey(),
            &self.rewards_mint_account.pubkey(),
        )
        .unwrap();
        //create user staking ata
        create_ata(
            rpc_client,
            &&self.payer,
            &user_wallet.pubkey(),
            &self.staking_mint_account.pubkey(),
        )
        .unwrap();
        //mint tokens 500 tokens to user staking ata to test staking pool
        mint_tokens_to(
            rpc_client,
            &self.payer,
            &self.staking_mint_account.pubkey(),
            &user_staking_ata,
            &self.owner_account,
            500u64 * TO_RAW_TOKEN,
        )
        .unwrap();
        let mut data = vec![1u8];
        data.push(bump_seed);

        airdrop_1_sol(rpc_client, &self.payer, &user_wallet.pubkey()).unwrap();
        let accounts = vec![
            AccountMeta::new(user_wallet.pubkey(), true),
            AccountMeta::new(user_storage_address, false),
            AccountMeta::new(self.pool_info_account.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ];

        let mut transaction = Transaction::new_with_payer(
            &[Instruction {
                program_id: self.program_id,
                accounts,
                data,
            }],
            Some(&self.payer.pubkey()),
        );
        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
        transaction.sign(&[&self.payer, user_wallet], recent_blockhash);
        rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|err| format!("error: send transaction: {}", err))
            .unwrap();
    }
    pub fn get_user_staking_ata(&self, user_wallet: &Pubkey) -> Pubkey {
        return spl_associated_token_account::get_associated_token_address(
            &user_wallet,
            &self.staking_mint_account.pubkey(),
        );
    }

    pub fn get_user_rewards_ata(&self, user_wallet: &Pubkey) -> Pubkey {
        return spl_associated_token_account::get_associated_token_address(
            &user_wallet,
            &self.rewards_mint_account.pubkey(),
        );
    }

    pub fn get_user_staking_token_balanace(
        &self,
        rpc_client: &mut RpcClient,
        user_wallet: &Pubkey,
    ) -> u64 {
        get_token_balance(rpc_client, &self.get_user_staking_ata(user_wallet))
    }

    pub fn get_user_rewards_token_balanace(
        &self,
        rpc_client: &mut RpcClient,
        user_wallet: &Pubkey,
    ) -> u64 {
        get_token_balance(rpc_client, &self.get_user_rewards_ata(user_wallet))
    }

    pub fn get_user_staking_token_balanace_ui(
        &self,
        rpc_client: &mut RpcClient,
        user_wallet: &Pubkey,
    ) -> f64 {
        self.get_user_staking_token_balanace(rpc_client, user_wallet) as f64 / TO_RAW_TOKEN as f64
    }

    pub fn get_user_rewards_token_balanace_ui(
        &self,
        rpc_client: &mut RpcClient,
        user_wallet: &Pubkey,
    ) -> f64 {
        self.get_user_rewards_token_balanace(rpc_client, user_wallet) as f64 / TO_RAW_TOKEN as f64
    }

    pub fn stake_cwar(
        &self,
        rpc_client: &mut RpcClient,
        user_wallet: &Keypair,
        amount_to_stake: u64,
        stake_after_seconds: u64,
    ) -> Result<(), TransportError> {
        println!(
            "Sleeping for {} seconds before stake cwar",
            stake_after_seconds
        );
        let stake_after_millis = Duration::from_millis(stake_after_seconds * 1000);
        thread::sleep(stake_after_millis);
        let user_storage_address = self.get_user_storage_address(&user_wallet.pubkey()).0;

        let user_staking_ata = self.get_user_staking_ata(&user_wallet.pubkey());

        let mut data = vec![2u8];
        data.extend(amount_to_stake.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(user_wallet.pubkey(), true),
            AccountMeta::new(user_storage_address, false),
            AccountMeta::new(self.pool_info_account.pubkey(), false),
            AccountMeta::new(self.staking_vault_account.pubkey(), false),
            AccountMeta::new(user_staking_ata, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];

        let mut transaction = Transaction::new_with_payer(
            &[Instruction {
                program_id: self.program_id,
                accounts,
                data,
            }],
            Some(&self.payer.pubkey()),
        );
        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
        transaction.sign(&[&self.payer, user_wallet], recent_blockhash);
        rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|err| format!("error: send transaction: {}", err))
            .unwrap();
        Ok(())
    }

    pub fn unstake_cwar(
        &self,
        rpc_client: &mut RpcClient,
        user_wallet: &Keypair,
        amount_to_unstake: u64,
        unstake_after_seconds: u64,
    ) -> Result<(), TransportError> {
        println!(
            "Sleeping for {} seconds before unstake cwar",
            unstake_after_seconds
        );
        let unstake_after_millis = Duration::from_millis(unstake_after_seconds * 1000);
        thread::sleep(unstake_after_millis);
        let user_storage_address = self.get_user_storage_address(&user_wallet.pubkey()).0;

        let user_staking_ata = self.get_user_staking_ata(&user_wallet.pubkey());

        let mut data = vec![3u8];
        data.extend(amount_to_unstake.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(user_wallet.pubkey(), true),
            AccountMeta::new(user_storage_address, false),
            AccountMeta::new(self.pool_info_account.pubkey(), false),
            AccountMeta::new(self.staking_vault_account.pubkey(), false),
            AccountMeta::new(user_staking_ata, false),
            AccountMeta::new_readonly(self.pool_signer_addr, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new(self.authority_unstake_penality_deposit_ata, false),
        ];

        let mut transaction = Transaction::new_with_payer(
            &[Instruction {
                program_id: self.program_id,
                accounts,
                data,
            }],
            Some(&self.payer.pubkey()),
        );
        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
        transaction.sign(&[&self.payer, user_wallet], recent_blockhash);
        rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|err| format!("error: send transaction: {}", err))
            .unwrap();
        Ok(())
    }

    pub fn print_pool_data(&self, rpc_client: &mut RpcClient) {
        let pool_data = self.get_pool_data(rpc_client);
        println!("Pool Onchain Data: {:?}", pool_data);
    }

    pub fn print_user_data(&self, rpc_client: &mut RpcClient, user_wallet: &Pubkey) {
        let user_data = self.get_user_data(rpc_client, user_wallet);
        println!("User Onchain Data: {:?}", user_data);
    }

    pub fn get_pool_data(&self, rpc_client: &mut RpcClient) -> CwarPool {
        let pool_data_u8_array = rpc_client
            .get_account_data(&self.pool_info_account.pubkey())
            .unwrap();
        CwarPool::try_from_slice(pool_data_u8_array.as_slice()).unwrap()
    }

    pub fn get_user_data(&self, rpc_client: &mut RpcClient, user_wallet: &Pubkey) -> User {
        let user_data_u8_array = rpc_client
            .get_account_data(&self.get_user_storage_address(user_wallet).0)
            .unwrap();
        User::try_from_slice(user_data_u8_array.as_slice()).unwrap()
    }

    pub fn get_pool_total_stake_last_update_timestamp(&self, rpc_client: &mut RpcClient) -> u64 {
        self.get_pool_data(rpc_client).total_stake_last_update_time
    }

    pub fn get_user_last_staked_timestamp(
        &self,
        rpc_client: &mut RpcClient,
        user_wallet: &Pubkey,
    ) -> u64 {
        self.get_user_data(rpc_client, user_wallet)
            .last_staked_timestamp
    }

    pub fn print_pool_total_stake_last_update_timestamp(&self, rpc_client: &mut RpcClient) {
        println!(
            "pool timestamp: {}",
            self.get_pool_data(rpc_client).total_stake_last_update_time
        );
    }

    pub fn print_user_last_staked_timestamp(
        &self,
        rpc_client: &mut RpcClient,
        user_wallet: &Pubkey,
    ) {
        println!(
            "user timestamp: {}",
            self.get_user_data(rpc_client, user_wallet)
                .last_staked_timestamp
        );
    }

    pub fn claim_rewards(&self, rpc_client: &mut RpcClient, user_wallet: &Keypair) {
        let user_storage_address = self.get_user_storage_address(&user_wallet.pubkey()).0;

        let user_rewards_ata = self.get_user_rewards_ata(&user_wallet.pubkey());

        let data = vec![4u8];

        let accounts = vec![
            AccountMeta::new(user_wallet.pubkey(), true),
            AccountMeta::new(user_storage_address, false),
            AccountMeta::new(self.pool_info_account.pubkey(), false),
            AccountMeta::new(self.staking_vault_account.pubkey(), false),
            AccountMeta::new(self.rewards_vault_account.pubkey(), false),
            AccountMeta::new(user_rewards_ata, false),
            AccountMeta::new_readonly(self.pool_signer_addr, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];

        let mut transaction = Transaction::new_with_payer(
            &[Instruction {
                program_id: self.program_id,
                accounts,
                data,
            }],
            Some(&self.payer.pubkey()),
        );
        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
        transaction.sign(&[&self.payer, user_wallet], recent_blockhash);
        rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|err| format!("error: send transaction: {}", err))
            .unwrap();
    }
}

//#[test]
#[allow(dead_code)]
fn test_claim_reward() {
    let pool = TestPool::new();

    let mut rpc_client = pool.test_validator.get_rpc_client();

    let start = Instant::now();
    println!("Time elapsed before init_pool: {:?}", start.elapsed());
    pool.init_pool(&mut rpc_client);
    println!("Time elapsed after init_pool: {:?}", start.elapsed());
    pool.fund_pool(&mut rpc_client);
    pool.print_pool_data(&mut rpc_client);
    println!("Time elapsed after fund_pool: {:?}", start.elapsed());
    // assert_eq!(
    //     get_token_balance(&mut rpc_client, &pool.rewards_vault_account.pubkey()).await,
    //     TO_RAW_TOKEN
    // );

    //     assert_eq!(pool_account_data.data.len(), CWAR_POOL_STORAGE_TOTAL_BYTES);
    //     assert_eq!(pool_account_data.owner, id());
    //pool.print_pool_data(&mut rpc_client);

    pool.create_user(&mut rpc_client, &pool.user_wallet1);
    println!("Time elapsed after create_user1: {:?}", start.elapsed());
    pool.print_user_data(&mut rpc_client, &pool.user_wallet1.pubkey());

    pool.create_user(&mut rpc_client, &pool.user_wallet2);
    println!("Time elapsed after create_user2: {:?}", start.elapsed());
    pool.print_user_data(&mut rpc_client, &pool.user_wallet2.pubkey());

    pool.print_pool_total_stake_last_update_timestamp(&mut rpc_client);
    pool.print_user_last_staked_timestamp(&mut rpc_client, &pool.user_wallet1.pubkey());
    pool.print_user_last_staked_timestamp(&mut rpc_client, &pool.user_wallet1.pubkey());
    // user1 stakes 100 at t = 7
    pool.stake_cwar(
        &mut rpc_client,
        &pool.user_wallet1,
        100u64 * TO_RAW_TOKEN,
        7u64,
    )
    .unwrap();
    println!("Time elapsed after stake_cwar1: {:?}", start.elapsed());
    pool.print_pool_total_stake_last_update_timestamp(&mut rpc_client);
    pool.print_user_last_staked_timestamp(&mut rpc_client, &pool.user_wallet1.pubkey());
    //pool.print_user_data(&mut rpc_client, &pool.user_wallet1.pubkey());

    // user2 stakes 100 at t = 9
    pool.stake_cwar(
        &mut rpc_client,
        &pool.user_wallet2,
        100u64 * TO_RAW_TOKEN,
        2u64,
    )
    .unwrap();
    println!("Time elapsed after stake_cwar2: {:?}", start.elapsed());
    pool.print_pool_total_stake_last_update_timestamp(&mut rpc_client);
    pool.print_user_last_staked_timestamp(&mut rpc_client, &pool.user_wallet2.pubkey());

    // user1 stakes 100 at t = 12
    pool.stake_cwar(
        &mut rpc_client,
        &pool.user_wallet1,
        100u64 * TO_RAW_TOKEN,
        3u64,
    )
    .unwrap();
    println!("Time elapsed after stake_cwar3: {:?}", start.elapsed());
    pool.print_pool_total_stake_last_update_timestamp(&mut rpc_client);
    pool.print_user_last_staked_timestamp(&mut rpc_client, &pool.user_wallet1.pubkey());

    // user1 unstakes 200 at t = 14
    pool.unstake_cwar(
        &mut rpc_client,
        &pool.user_wallet1,
        200u64 * TO_RAW_TOKEN,
        2u64,
    )
    .unwrap();

    println!("Time elapsed after unstake_cwar1: {:?}", start.elapsed());
    pool.print_pool_total_stake_last_update_timestamp(&mut rpc_client);
    pool.print_user_last_staked_timestamp(&mut rpc_client, &pool.user_wallet1.pubkey());
    // user2 unstakes 100 at t = 18
    pool.unstake_cwar(
        &mut rpc_client,
        &pool.user_wallet2,
        100u64 * TO_RAW_TOKEN,
        4u64,
    )
    .unwrap();
    println!("Time elapsed after unstake_cwar2: {:?}", start.elapsed());

    let duration = start.elapsed();

    println!("Time elapsed in test claim rewards is: {:?}", duration);

    assert_eq!(
        pool.get_user_rewards_token_balanace(&mut rpc_client, &pool.user_wallet1.pubkey()),
        0
    );
    assert_eq!(
        pool.get_user_rewards_token_balanace(&mut rpc_client, &pool.user_wallet2.pubkey()),
        0
    );
    pool.claim_rewards(&mut rpc_client, &pool.user_wallet1);
    pool.claim_rewards(&mut rpc_client, &pool.user_wallet2);

    thread::sleep(Duration::from_millis(5 * 1000));
    println!(
        "user1 rewards token balance: {}",
        pool.get_user_rewards_token_balanace_ui(&mut rpc_client, &pool.user_wallet1.pubkey())
    );
    println!(
        "user2 rewards token balance: {}",
        pool.get_user_rewards_token_balanace_ui(&mut rpc_client, &pool.user_wallet2.pubkey())
    );
}
