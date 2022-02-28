import {Commitment, Connection, Keypair, Transaction} from '@solana/web3.js';
import {ConnectionService} from '../config';

export const NUMBER_OF_RETRIES = 3;

export async function sendTransactionWithRetries(
  transaction: Transaction,
  signers?: Keypair[],
  commitment?: Commitment
): Promise<string> {
  const connection = ConnectionService.getConnection();
  for (let i = 0; i < NUMBER_OF_RETRIES; i++) {
    try {
      let signature;
      if (!signers) {
        signature = await connection.sendRawTransaction(
          transaction.serialize()
        );
      } else {
        signature = await connection.sendTransaction(transaction, signers);
      }

      await confirmTransactionWithRetries(connection, signature, commitment);
      return signature;
    } catch (err) {
      console.error(err);
    }
  }

  throw new Error('Max Retries Hit');
}

export async function confirmTransactionWithRetries(
  connection: Connection,
  signature: string,
  commitment?: Commitment
): Promise<void> {
  if (!commitment) commitment = 'processed';

  for (let i = 0; i < NUMBER_OF_RETRIES; i++) {
    try {
      const result = await connection.confirmTransaction(signature, commitment);
      if (!result || result?.value?.err) {
        continue; // Try again
      }
    } catch (err) {
      continue; // Try again
    }

    return;
  }

  throw new Error('Unable to confirm transaction');
}
