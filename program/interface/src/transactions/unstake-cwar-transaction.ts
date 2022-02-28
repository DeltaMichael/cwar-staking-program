import {PublicKey, Transaction, TransactionInstruction} from '@solana/web3.js';
import {
  findAssociatedTokenAddress,
  getPoolSignerPDA,
  getUserStorageAccount,
} from '../utils';
import {Constants, Pubkeys} from '../constants';
import {ConnectionService} from '../config';
import {CwarStakingInstructions} from '../models';
import BN from 'bn.js';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';

export async function unstakeCwarTransaction(
  userWallet: PublicKey,
  amountToWithdraw: number
): Promise<Transaction> {
  const connection = ConnectionService.getConnection();

  const userStoragePubkey = await getUserStorageAccount(userWallet);

  const stakingATAPubkey = await findAssociatedTokenAddress(
    userWallet,
    Pubkeys.stakingMintPubkey
  );

  const amountToWithdrawRaw = new BN(
    amountToWithdraw * Constants.decimalPrecision
  ).mul(new BN(Constants.toCwarRaw / Constants.decimalPrecision));

  const poolSignerPda = await getPoolSignerPDA();

  const unstakeCwarIxs: TransactionInstruction[] = [];
  const unstakeCwarIx = new TransactionInstruction({
    programId: Pubkeys.cwarStakingProgramId,
    keys: [
      {
        pubkey: userWallet,
        isSigner: true,
        isWritable: false,
      },

      {
        pubkey: userStoragePubkey,
        isSigner: false,
        isWritable: true,
      },

      {
        pubkey: Pubkeys.cwarPoolStoragePubkey,
        isSigner: false,
        isWritable: true,
      },

      {
        pubkey: Pubkeys.cwarStakingVaultPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: stakingATAPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: poolSignerPda,
        isSigner: false,
        isWritable: false,
      },
      {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false},
      {
        pubkey: Pubkeys.unstakePenalityATAPubkey,
        isSigner: false,
        isWritable: true,
      },
    ],
    data: Buffer.from([
      CwarStakingInstructions.UnstakeCwar,
      ...amountToWithdrawRaw.toArray('le', 8),
    ]),
  });
  const rewardsATAPubkey = await findAssociatedTokenAddress(
    userWallet,
    Pubkeys.rewardsMintPubkey
  );
  const rewardsAtaInfo = await connection.getAccountInfo(rewardsATAPubkey);

  const doesRewardsAtaExist = rewardsAtaInfo?.owner !== undefined;

  if (!doesRewardsAtaExist) {
    const createFantAssociatedAccountIx =
      Token.createAssociatedTokenAccountInstruction(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        Pubkeys.rewardsMintPubkey,
        rewardsATAPubkey,
        userWallet,
        userWallet
      );
    unstakeCwarIxs.push(createFantAssociatedAccountIx);
  }

  const claimRewardsIx = new TransactionInstruction({
    programId: Pubkeys.cwarStakingProgramId,
    keys: [
      {
        pubkey: userWallet,
        isSigner: true,
        isWritable: false,
      },

      {
        pubkey: userStoragePubkey,
        isSigner: false,
        isWritable: true,
      },

      {
        pubkey: Pubkeys.cwarPoolStoragePubkey,
        isSigner: false,
        isWritable: true,
      },

      {
        pubkey: Pubkeys.cwarStakingVaultPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: Pubkeys.cwarRewardsVaultPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: rewardsATAPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: poolSignerPda,
        isSigner: false,
        isWritable: false,
      },
      {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false},
    ],
    data: Buffer.from([CwarStakingInstructions.ClaimRewards]),
  });

  unstakeCwarIxs.push(unstakeCwarIx);
  unstakeCwarIxs.push(claimRewardsIx);

  const unstakeCwarTx = new Transaction().add(...unstakeCwarIxs);
  unstakeCwarTx.recentBlockhash = (
    await connection.getRecentBlockhash()
  ).blockhash;
  unstakeCwarTx.feePayer = userWallet;

  return unstakeCwarTx;
}
