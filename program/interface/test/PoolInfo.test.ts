import {Keypair, PublicKey} from '@solana/web3.js';
import BN from 'bn.js';
import {Constants} from '../src/constants';
import {CwarPoolData} from '../src/models';

describe('Pool Info model', () => {
  test('serializes to buffer of correct size and data', () => {
    const accType = 2;
    const ownerWallet: string = Keypair.generate().publicKey.toString();
    const stakingVault: string = Keypair.generate().publicKey.toString();
    const stakingMint: string = Keypair.generate().publicKey.toString();
    const rewardVault: string = Keypair.generate().publicKey.toString();
    const rewardMint: string = Keypair.generate().publicKey.toString();
    const rewardRate: BN = new BN(1_234_456_789);
    const rewardDuration: BN = new BN(3_839_754_739);
    const totalStakeLastUpdateTime: BN = new BN(5_123_484_888);
    const rewardPerTokenStored: BN = new BN(7_212_589_757).mul(
      new BN(Constants.u64MaxStrValue)
    );
    const userStakeCount = 923_439_986;
    const pdaNonce = 247;
    const funders: string[] = [
      Keypair.generate().publicKey.toString(),
      Keypair.generate().publicKey.toString(),
      Keypair.generate().publicKey.toString(),
      Keypair.generate().publicKey.toString(),
      Keypair.generate().publicKey.toString(),
    ];
    const rewardDurationEnd: BN = new BN(4_384_855_745);
    const unstakePenalityBasisPoints = 342;
    const lockingDuration: BN = new BN(6_854_437_474);
    const authorityPenalityDepositATA: string =
      Keypair.generate().publicKey.toString();
    const poolInfo = new CwarPoolData({
      accountType: accType,
      ownerWallet,
      stakingVault,
      stakingMint,
      rewardVault,
      rewardMint,
      rewardRate,
      rewardDuration,
      totalStakeLastUpdateTime,
      rewardPerTokenStored,
      userStakeCount,
      pdaNonce,
      funders,
      rewardDurationEnd,
      unstakePenalityBasisPoints,
      lockingDuration,
      authorityPenalityDepositATA,
    });
    const buffer = poolInfo.toBuffer();
    expect(buffer instanceof Buffer).toBe(true);
    expect(buffer.byteLength).toBe(416);

    const poolOnChainData = CwarPoolData.fromBuffer(buffer);
    //expect(poolOnChainData).toMatchObject(poolInfo);
    expect(poolOnChainData.accountType).toBe(accType);

    expect(poolOnChainData.ownerWallet).toBe(ownerWallet);
    expect(poolOnChainData.getAuthorityPubkey()).toStrictEqual(
      new PublicKey(ownerWallet)
    );

    expect(poolOnChainData.stakingVault).toBe(stakingVault);
    expect(poolOnChainData.getStakingVaultPubkey()).toStrictEqual(
      new PublicKey(stakingVault)
    );

    expect(poolOnChainData.stakingMint).toBe(stakingMint);
    expect(poolOnChainData.getStakingMintPubkey()).toStrictEqual(
      new PublicKey(stakingMint)
    );

    expect(poolOnChainData.rewardVault).toBe(rewardVault);
    expect(poolOnChainData.getRewardVaultPubkey()).toStrictEqual(
      new PublicKey(rewardVault)
    );

    expect(poolOnChainData.rewardMint).toBe(rewardMint);
    expect(poolOnChainData.getRewardMintPubkey()).toStrictEqual(
      new PublicKey(rewardMint)
    );
    expect(poolOnChainData.rewardRate.toNumber()).toBe(rewardRate.toNumber());
    expect(poolOnChainData.getRewardRate()).toBe(1.234_456);

    expect(poolOnChainData.rewardDuration.toNumber()).toBe(
      rewardDuration.toNumber()
    );
    expect(poolOnChainData.getRewardDuration()).toBe(3_839_754_739);

    expect(poolOnChainData.totalStakeLastUpdateTime.toNumber()).toBe(
      totalStakeLastUpdateTime.toNumber()
    );
    expect(poolOnChainData.getTotalStakeLastUpdateTime()).toBe(5_123_484_888);

    expect(
      poolOnChainData.rewardPerTokenStored
        .div(new BN(Constants.u64MaxStrValue))
        .toNumber()
    ).toBe(
      rewardPerTokenStored.div(new BN(Constants.u64MaxStrValue)).toNumber()
    );
    expect(poolOnChainData.getRewardPerTokenStored()).toBe(7.212_589);

    expect(poolOnChainData.userStakeCount).toBe(userStakeCount);
    expect(poolOnChainData.getUserStakeCount()).toBe(923_439_986);

    expect(poolOnChainData.pdaNonce).toBe(pdaNonce);
    expect(poolOnChainData.getPdaNonce()).toBe(247);

    expect(poolOnChainData.funders).toStrictEqual(funders);
    expect(poolOnChainData.getFundersArray()).toStrictEqual(funders);

    expect(poolOnChainData.rewardDurationEnd.toNumber()).toBe(
      rewardDurationEnd.toNumber()
    );
    expect(poolOnChainData.getRewardDurationEnd()).toBe(4_384_855_745);

    expect(poolOnChainData.unstakePenalityBasisPoints).toBe(
      unstakePenalityBasisPoints
    );
    expect(poolOnChainData.getUnstakePenalityBasisPoints()).toBe(342);

    expect(poolOnChainData.lockingDuration.toNumber()).toBe(
      lockingDuration.toNumber()
    );
    expect(poolOnChainData.getLockingDuration()).toBe(6_854_437_474);

    expect(poolOnChainData.authorityPenalityDepositATA).toBe(
      authorityPenalityDepositATA
    );
    expect(poolOnChainData.getAuthorityPenalityDepositATA()).toStrictEqual(
      new PublicKey(authorityPenalityDepositATA)
    );
  });
});
