import {PublicKey, Transaction, TransactionInstruction} from '@solana/web3.js';

import {ConnectionService} from '../config';
import {Pubkeys} from '../constants';
import {CwarStakingInstructions} from '../models';

export async function removeFunderTransaction(
  poolOwnerWallet: PublicKey,
  funderToRemove: PublicKey
): Promise<Transaction> {
  const connection = ConnectionService.getConnection();

  const removeFunderIx = new TransactionInstruction({
    programId: Pubkeys.cwarStakingProgramId,
    keys: [
      {
        pubkey: poolOwnerWallet,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: Pubkeys.cwarPoolStoragePubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: funderToRemove,
        isSigner: false,
        isWritable: false,
      },
    ],
    data: Buffer.from([CwarStakingInstructions.RemoveFunder]),
  });

  const removeFunderTx = new Transaction().add(removeFunderIx);

  removeFunderTx.recentBlockhash = (
    await connection.getRecentBlockhash()
  ).blockhash;
  removeFunderTx.feePayer = poolOwnerWallet;

  return removeFunderTx;
}
