import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  Transaction,
} from '@solana/web3.js';
import {ConnectionService, SolanaNet} from '../src/config';
import {getKeyPair} from '../scripts/get-public-key';
import {Constants, Pubkeys} from '../src/constants';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
  u64,
} from '@solana/spl-token';
import {findAssociatedTokenAddress} from '../src/utils';
import BN from 'bn.js';

export function setupTest(): void {
  jest.setTimeout(6_000_000);
  ConnectionService.setNet(SolanaNet.LOCALNET);
}

export async function requestAirdrop(publicKey: PublicKey): Promise<void> {
  const connection = ConnectionService.getConnection();
  const airdropTxSig = await connection.requestAirdrop(
    publicKey,
    LAMPORTS_PER_SOL
  );
  await connection.confirmTransaction(airdropTxSig, 'processed');
}

export function timeout(ms): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function getAdminKeypair(): Keypair {
  return getKeyPair('../admin-keypair.json');
}

export async function findAndCreateRewardAta(
  wallet: Keypair
): Promise<PublicKey> {
  const walletAta = await findAndCreateAta(wallet, Pubkeys.rewardsMintPubkey);
  return walletAta;
}

export async function findAndCreateStakingAta(
  wallet: Keypair
): Promise<PublicKey> {
  const walletAta = await findAndCreateAta(wallet, Pubkeys.stakingMintPubkey);
  return walletAta;
}

export async function getTokenBalance(
  tokenAddress: PublicKey
): Promise<number> {
  const connection = ConnectionService.getConnection();
  const tokenBalance = await connection.getTokenAccountBalance(tokenAddress);
  return tokenBalance.value.uiAmount as number;
}

export async function getSolBalance(address: PublicKey): Promise<number> {
  const connection = ConnectionService.getConnection();
  const tokenBalance = await connection.getBalance(address);
  return tokenBalance;
}

export async function findAndCreateAta(
  userWallet: Keypair,
  mint: PublicKey
): Promise<PublicKey> {
  const userAta = await findAssociatedTokenAddress(userWallet.publicKey, mint);
  const connection = ConnectionService.getConnection();
  const userAtaInfo = await connection.getAccountInfo(userAta);

  const doesUserAtaExist = userAtaInfo?.owner !== undefined;

  if (!doesUserAtaExist) {
    const createUserAtaIx = Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      mint,
      userAta,
      userWallet.publicKey,
      userWallet.publicKey
    );
    const createUserAtaTx = new Transaction().add(createUserAtaIx);
    await sendAndConfirmTransaction(connection, createUserAtaTx, [userWallet]);
  }
  return userAta;
}

export async function mintTokensTo(
  tokenMint: Token,
  dstWallet: Keypair,
  authority: PublicKey,
  amount: number
): Promise<void> {
  if (!amount) amount = 1_000;
  // Mint 1000 tokens by default to userWallet ATA of Token Mint
  const mintInfo = await tokenMint.getMintInfo();

  const tokensToMint = new BN(amount * Constants.decimalPrecision).mul(
    new BN(Math.pow(10, mintInfo.decimals) / Constants.decimalPrecision)
  );
  const tokensToMintBuf = tokensToMint.toBuffer('le', 8);
  const dstWalletAta = await findAndCreateAta(dstWallet, tokenMint.publicKey);
  await tokenMint.mintTo(
    dstWalletAta,
    authority,
    [],
    u64.fromBuffer(tokensToMintBuf)
  );
}
