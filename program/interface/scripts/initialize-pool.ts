import {getAdminKeypair, requestAirdrop} from '../test/testHelpers';
import {
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  Transaction,
} from '@solana/web3.js';
import {
  createInitializePoolTransaction,
  fundPoolTransaction,
} from '../src/transactions';
import {ConnectionService} from '../src/config';
import {Constants, Pubkeys} from '../src/constants';
import {
  Token,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  u64,
} from '@solana/spl-token';
import {findAssociatedTokenAddress} from '../src/utils';
import BN from 'bn.js';

const adminKeypair: Keypair = getAdminKeypair();
let cwarPoolStorageAccount: Keypair;
let cwarStakingVault: Keypair;
let cwarRewardsVault: Keypair;
const stakingTokenDecimals = 9;
const rewardTokenDecimals = 9;
let unstakePenalityPercentage: number;
let lockingPeriodDurationDays: number;

export enum SolanaNet {
  MAINNET = 'mainnet',
  DEVNET = 'devnet',
  LOCALNET = 'localnet',
}

export const initPoolAndUser = async (userKey: string) => {
  ConnectionService.setNet(SolanaNet.DEVNET);
  Constants.stakingTokenDecimals = stakingTokenDecimals;
  Constants.rewardTokenDecimals = rewardTokenDecimals;

  const connection = ConnectionService.getConnection();

  // Create Cwar Test Token
  const stakingTokenMint = await Token.createMint(
    connection,
    adminKeypair,
    adminKeypair.publicKey,
    null,
    stakingTokenDecimals,
    TOKEN_PROGRAM_ID
  );
  Pubkeys.stakingMintPubkey = stakingTokenMint.publicKey;

  console.log(`Cwar mint pubkey: ${Pubkeys.stakingMintPubkey}`);
  console.log(`Staking mint pubkey: ${Pubkeys.stakingMintPubkey}`);
  // Create Reward Test Token
  const rewardTokenMint = await Token.createMint(
    connection,
    adminKeypair,
    adminKeypair.publicKey,
    null,
    rewardTokenDecimals,
    TOKEN_PROGRAM_ID
  );
  Pubkeys.rewardsMintPubkey = rewardTokenMint.publicKey;
  console.log(`Rewards mint pubkey: ${Pubkeys.rewardsMintPubkey}`);
  cwarPoolStorageAccount = Keypair.generate();
  cwarStakingVault = Keypair.generate();
  cwarRewardsVault = Keypair.generate();
  unstakePenalityPercentage = 0;
  lockingPeriodDurationDays = 0;
  // await requestAirdrop(adminKeypair.publicKey);
  await new Promise(f => setTimeout(f, 10000));
  const initializePoolTx = await createInitializePoolTransaction(
    adminKeypair.publicKey,
    cwarPoolStorageAccount,
    cwarStakingVault,
    cwarRewardsVault,
    unstakePenalityPercentage,
    lockingPeriodDurationDays
  );
  await sendAndConfirmTransaction(connection, initializePoolTx, [
    adminKeypair,
    cwarPoolStorageAccount,
    cwarStakingVault,
    cwarRewardsVault,
  ]);
  await new Promise(f => setTimeout(f, 10000));

  Pubkeys.cwarPoolStoragePubkey = cwarPoolStorageAccount.publicKey;
  Pubkeys.cwarStakingVaultPubkey = cwarStakingVault.publicKey;
  Pubkeys.cwarRewardsVaultPubkey = cwarRewardsVault.publicKey;
  Pubkeys.unstakePenalityATAPubkey = await findAssociatedTokenAddress(
    adminKeypair.publicKey,
    Pubkeys.stakingMintPubkey
  );

  const funderRewardTokenAta = await findAssociatedTokenAddress(
    adminKeypair.publicKey,
    Pubkeys.rewardsMintPubkey
  );

  const funderRewardAtaInfo = await connection.getAccountInfo(
    funderRewardTokenAta
  );

  const doesRewardsAtaExist = funderRewardAtaInfo?.owner !== undefined;

  if (!doesRewardsAtaExist) {
    const createFunderRewardsAtaIx =
      Token.createAssociatedTokenAccountInstruction(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        Pubkeys.rewardsMintPubkey,
        funderRewardTokenAta,
        adminKeypair.publicKey,
        adminKeypair.publicKey
      );
    const createFunderRewardsAtaTx = new Transaction().add(
      createFunderRewardsAtaIx
    );
    await sendAndConfirmTransaction(connection, createFunderRewardsAtaTx, [
      adminKeypair,
    ]);
    await new Promise(f => setTimeout(f, 10000));
  }

  // Mint 10000 Reward token to funder wallet for fund pool
  const rewardsTokenToMint = 10000;
  const rewardTokensToMintRaw = new BN(rewardsTokenToMint)
    .mul(new BN(Constants.toCwarRaw))
    .toArray('le', 8);
  await rewardTokenMint.mintTo(
    funderRewardTokenAta,
    adminKeypair.publicKey,
    [],
    new u64(rewardTokensToMintRaw)
  );
  await new Promise(f => setTimeout(f, 10000));
  await fundUser(userKey, stakingTokenMint);
};

export const fundPool = async () => {
  const connection = ConnectionService.getConnection();
  const amountToFundPool = 10000;
  const extensionInDays = 0;
  // fund pool
  const fundPoolTx = await fundPoolTransaction(
    adminKeypair.publicKey,
    amountToFundPool,
    extensionInDays
  );
  await sendAndConfirmTransaction(connection, fundPoolTx, [adminKeypair]);
  await new Promise(f => setTimeout(f, 10000));
};

export const fundUser = async (userPublicKey: string, mint: Token) => {
  ConnectionService.setNet(SolanaNet.DEVNET);
  const key = new PublicKey(userPublicKey);
  await requestAirdrop(key);
  const connection = ConnectionService.getConnection();
  const userStakingTokenAta = await findAssociatedTokenAddress(
    key,
    Pubkeys.stakingMintPubkey
  );

  const userStakingTokenAtaInfo = await connection.getAccountInfo(
    userStakingTokenAta
  );

  const doesStakingAtaExist = userStakingTokenAtaInfo?.owner !== undefined;

  if (!doesStakingAtaExist) {
    const createUserStakingAtaIx =
      Token.createAssociatedTokenAccountInstruction(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        Pubkeys.stakingMintPubkey,
        userStakingTokenAta,
        key,
        adminKeypair.publicKey
      );
    const createUserStakingAtaTx = new Transaction().add(
      createUserStakingAtaIx
    );
    await sendAndConfirmTransaction(connection, createUserStakingAtaTx, [
      adminKeypair,
    ]);
    await new Promise(f => setTimeout(f, 10000));
  }

  // Mint 10000 Reward token to funder wallet for fund pool
  const rewardsTokenToMint = 10000;
  const rewardTokensToMintRaw = new BN(rewardsTokenToMint)
    .mul(new BN(Constants.toCwarRaw))
    .toArray('le', 8);
  await mint.mintTo(
    userStakingTokenAta,
    adminKeypair.publicKey,
    [],
    new u64(rewardTokensToMintRaw)
  );
};

if (require.main === module) {
  (async () => {
    await initPoolAndUser(process.argv[2]);
    await fundPool();
  })();
}
