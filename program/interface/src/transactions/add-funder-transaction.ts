import {PublicKey, Transaction, TransactionInstruction} from '@solana/web3.js';

import {ConnectionService} from '../config';
import {Pubkeys} from '../constants';
import {CwarStakingInstructions} from '../models';

export async function addFunderTransaction(
  poolOwnerWallet: PublicKey,
  funderToAdd: PublicKey
): Promise<Transaction> {
  const connection = ConnectionService.getConnection();

  const addFunderIx = new TransactionInstruction({
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
        pubkey: funderToAdd,
        isSigner: false,
        isWritable: false,
      },
    ],
    data: Buffer.from([CwarStakingInstructions.AddFunder]),
  });

  const addFunderTx = new Transaction().add(addFunderIx);

  addFunderTx.recentBlockhash = (
    await connection.getRecentBlockhash()
  ).blockhash;
  addFunderTx.feePayer = poolOwnerWallet;

  return addFunderTx;
}
