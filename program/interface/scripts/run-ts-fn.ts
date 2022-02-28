import {Token, TOKEN_PROGRAM_ID, u64} from '@solana/spl-token';
import {Keypair, LAMPORTS_PER_SOL} from '@solana/web3.js';
import BN from 'bn.js';
import {ConnectionService, SolanaNet} from '../src/config';
import {Constants, Pubkeys} from '../src/constants';
import {findAndCreateAta, getAdminKeypair} from '../test/testHelpers';

ConnectionService.setNet(SolanaNet.LOCALNET);

async function runTsFn(
  rewardTokenAmountToFundPoolWith?: number
): Promise<void> {
  const connection = ConnectionService.getConnection();
  const adminKeypair = getAdminKeypair();
  const airdropTxSig = await connection.requestAirdrop(
    adminKeypair.publicKey,
    LAMPORTS_PER_SOL
  );
  const funderWallet = Keypair.generate();
  await connection.confirmTransaction(airdropTxSig, 'processed');
  const airdropTxSig2 = await connection.requestAirdrop(
    funderWallet.publicKey,
    LAMPORTS_PER_SOL
  );
  await connection.confirmTransaction(airdropTxSig2, 'processed');
  // Create Reward Test Token
  const rewardTokenMint = await Token.createMint(
    connection,
    adminKeypair,
    adminKeypair.publicKey,
    adminKeypair.publicKey,
    9,
    TOKEN_PROGRAM_ID
  );
  Pubkeys.rewardsMintPubkey = rewardTokenMint.publicKey;

  if (!rewardTokenAmountToFundPoolWith)
    rewardTokenAmountToFundPoolWith = 10_000;
  console.log(
    'rewardTokenAmountToFundPoolWith: ',
    rewardTokenAmountToFundPoolWith
  );
  // Mint 10000 Reward token by default to funder wallet to fund pool for testing
  // const rewardTokensToMintRaw = new BN(rewardsTokenToMint)
  //   .mul(new BN(Constants.toRewardTokenRaw))
  //   .toArray('le', 8);
  const rewardTokensToMintRaw = new BN(rewardTokenAmountToFundPoolWith).mul(
    new BN(Constants.toRewardTokenRaw)
  );
  const rewardTokensToMintRawBuf = rewardTokensToMintRaw.toBuffer('le', 8);
  const funderRewardAta = await findAndCreateAta(
    funderWallet,
    Pubkeys.rewardsMintPubkey
  );
  const funderRewardTokenBalanceBefore =
    await connection.getTokenAccountBalance(funderRewardAta);
  console.log(
    'funderRewardTokenBalanceBefore: ',
    funderRewardTokenBalanceBefore.value.uiAmount
  );
  await rewardTokenMint.mintTo(
    funderRewardAta,
    adminKeypair.publicKey,
    [],
    u64.fromBuffer(rewardTokensToMintRawBuf)
  );

  const funderRewardTokenBalanceAfter = await connection.getTokenAccountBalance(
    funderRewardAta
  );
  console.log(
    'funderRewardTokenBalanceAfter: ',
    funderRewardTokenBalanceAfter.value.uiAmount
  );
}

runTsFn(10_000_000);
