import {
  AccountLayout,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from '@solana/web3.js';
import BN from 'bn.js';

import {ConnectionService} from '../config';
import {Constants, Pubkeys} from '../constants';
import {CwarStakingInstructions} from '../models';
import {findAssociatedTokenAddress, getPoolSignerPdaNonce} from '../utils';

export async function createInitializePoolTransaction(
  poolOwnerWallet: PublicKey,
  cwarPoolStorageAccount: Keypair,
  cwarStakingVault: Keypair,
  cwarRewardsVault: Keypair,
  unstakePenalityPercentage: number,
  lockingDurationInDays: number
): Promise<Transaction> {
  const connection = ConnectionService.getConnection();
  const poolStorageBytes = Constants.cwarPoolBytes;
  const rewardDurationInDays = 0;
  const rewardDuration = rewardDurationInDays * Constants.secondsInOneDay;
  const unstakePenalityBasisPoints = unstakePenalityPercentage * 100;
  const lockingDuration = lockingDurationInDays * Constants.secondsInOneDay;
  console.log(
    'Pool Storage Pubkey: ',
    cwarPoolStorageAccount.publicKey.toString()
  );
  console.log('Staking Vault Pubkey: ', cwarStakingVault.publicKey.toString());
  console.log('Rewards Vault Pubkey: ', cwarRewardsVault.publicKey.toString());
  const createStakingVaultIx = SystemProgram.createAccount({
    space: AccountLayout.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      AccountLayout.span,
      'confirmed'
    ),
    fromPubkey: poolOwnerWallet,
    newAccountPubkey: cwarStakingVault.publicKey,
    programId: TOKEN_PROGRAM_ID,
  });

  const initStakingVaultIx = Token.createInitAccountInstruction(
    TOKEN_PROGRAM_ID,
    Pubkeys.stakingMintPubkey,
    cwarStakingVault.publicKey,
    poolOwnerWallet
  );
  const createRewardsVaultIx = SystemProgram.createAccount({
    space: AccountLayout.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      AccountLayout.span,
      'confirmed'
    ),
    fromPubkey: poolOwnerWallet,
    newAccountPubkey: cwarRewardsVault.publicKey,
    programId: TOKEN_PROGRAM_ID,
  });

  const initRewardsVaultIx = Token.createInitAccountInstruction(
    TOKEN_PROGRAM_ID,
    Pubkeys.rewardsMintPubkey,
    cwarRewardsVault.publicKey,
    poolOwnerWallet
  );
  const pool_nonce = await getPoolSignerPdaNonce();
  const rentPrice = await connection.getMinimumBalanceForRentExemption(
    poolStorageBytes,
    'confirmed'
  );
  const createPoolStorageAccountIx = SystemProgram.createAccount({
    space: poolStorageBytes,
    lamports: rentPrice,
    fromPubkey: poolOwnerWallet,
    newAccountPubkey: cwarPoolStorageAccount.publicKey,
    programId: Pubkeys.cwarStakingProgramId,
  });

  const balance = await connection.getBalance(poolOwnerWallet);
  if (balance < rentPrice)
    throw new Error(
      `Need at least ${
        rentPrice / LAMPORTS_PER_SOL
      } SOL for contest account rent`
    );

  const unstakePenalityATAPubkey = await findAssociatedTokenAddress(
    poolOwnerWallet,
    Pubkeys.stakingMintPubkey
  );

  console.log('Unstake Penality ATA: ', unstakePenalityATAPubkey.toString());

  const initPoolStorageAccountIx = new TransactionInstruction({
    programId: Pubkeys.cwarStakingProgramId,
    keys: [
      {
        pubkey: poolOwnerWallet,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: cwarPoolStorageAccount.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: Pubkeys.stakingMintPubkey,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: cwarStakingVault.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: Pubkeys.rewardsMintPubkey,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: cwarRewardsVault.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: unstakePenalityATAPubkey,
        isSigner: false,
        isWritable: false,
      },
    ],
    data: Buffer.from([
      CwarStakingInstructions.InitializeCwarPool,
      ...new BN(rewardDuration).toArray('le', 8),
      ...new BN(pool_nonce.valueOf()).toArray('le', 1),
      ...new BN(unstakePenalityBasisPoints).toArray('le', 2),
      ...new BN(lockingDuration).toArray('le', 8),
    ]),
  });

  const unstakePenalityAtaInfo = await connection.getAccountInfo(
    unstakePenalityATAPubkey
  );

  const doesUnstakePenalityAtaExist =
    unstakePenalityAtaInfo?.owner !== undefined;
  const createPenalityATAIxs: TransactionInstruction[] = [];
  if (!doesUnstakePenalityAtaExist) {
    const createUnstakePenalityATAIx =
      Token.createAssociatedTokenAccountInstruction(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        Pubkeys.stakingMintPubkey,
        unstakePenalityATAPubkey,
        poolOwnerWallet,
        poolOwnerWallet
      );
    createPenalityATAIxs.push(createUnstakePenalityATAIx);
  }

  const transaction = new Transaction().add(
    ...createPenalityATAIxs,
    createStakingVaultIx,
    initStakingVaultIx,
    createRewardsVaultIx,
    initRewardsVaultIx,
    createPoolStorageAccountIx,
    initPoolStorageAccountIx
  );

  transaction.recentBlockhash = (
    await connection.getRecentBlockhash()
  ).blockhash;
  transaction.feePayer = poolOwnerWallet;

  transaction.partialSign(
    cwarStakingVault,
    cwarRewardsVault,
    cwarPoolStorageAccount
  );

  return transaction;
}
