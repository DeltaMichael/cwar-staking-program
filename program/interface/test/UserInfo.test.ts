import {Keypair, PublicKey} from '@solana/web3.js';
import BN from 'bn.js';
import {Constants} from '../src/constants';
import {UserData} from '../src/models';

describe('User Info model', () => {
  test('serializes to buffer of correct size and data', () => {
    const accType = 3;
    const userWallet: string = Keypair.generate().publicKey.toString();
    const cwarPool: string = Keypair.generate().publicKey.toString();
    const userCwarStakedAmount: BN = new BN(1_234_456_789);
    const nonce = 235;
    const rewardsAmountPending: BN = new BN(5_123_484_888);
    const rewardsPerTokenCompleted: BN = new BN(7_212_589_757).mul(
      new BN(Constants.u64MaxStrValue)
    );
    const unstakePenalityDurationEnd: BN = new BN(4_384_855_745);
    const lastStakedTimestamp: BN = new BN(6_854_437_474);
    const userInfo = new UserData({
      accountType: accType,
      userWallet,
      cwarPool,
      balanceStaked: userCwarStakedAmount,
      nonce,
      rewardPerTokenPending: rewardsAmountPending,
      rewardsPerTokenCompleted,
      unstakePenalityDurationEnd,
      lastStakedTimestamp,
    });
    const buffer = userInfo.toBuffer();
    expect(buffer instanceof Buffer).toBe(true);
    expect(buffer.byteLength).toBe(114);

    const userOnChainData = UserData.fromBuffer(buffer);
    //expect(userOnChainData).toMatchObject(userInfo);
    expect(userOnChainData.accountType).toBe(accType);

    expect(userOnChainData.userWallet).toBe(userWallet);
    expect(userOnChainData.getUserWalletPubkey()).toStrictEqual(
      new PublicKey(userWallet)
    );

    expect(userOnChainData.cwarPool).toBe(cwarPool);
    expect(userOnChainData.getPoolPubkey()).toStrictEqual(
      new PublicKey(cwarPool)
    );

    expect(userOnChainData.balanceStaked.toNumber()).toBe(
      userCwarStakedAmount.toNumber()
    );
    expect(userOnChainData.getBalanceStaked()).toStrictEqual(1.234456);

    expect(userOnChainData.nonce).toBe(nonce);
    expect(userOnChainData.getNonce()).toBe(235);

    expect(userOnChainData.rewardPerTokenPending.toNumber()).toBe(
      rewardsAmountPending.toNumber()
    );
    expect(userOnChainData.getRewardPerTokenPending()).toBe(5.123484);

    expect(
      userOnChainData.rewardsPerTokenCompleted
        .div(new BN(Constants.u64MaxStrValue))
        .toNumber()
    ).toBe(
      rewardsPerTokenCompleted.div(new BN(Constants.u64MaxStrValue)).toNumber()
    );
    expect(userOnChainData.getRewardPerTokenCompleted()).toBe(7.212_589);

    expect(userOnChainData.unstakePenalityDurationEnd.toNumber()).toBe(
      unstakePenalityDurationEnd.toNumber()
    );
    expect(userOnChainData.getUnstakePenalityDurationEnd()).toBe(4_384_855_745);

    expect(userOnChainData.lastStakedTimestamp.toNumber()).toBe(
      lastStakedTimestamp.toNumber()
    );
    expect(userOnChainData.getLastStakedTimestamp()).toBe(6_854_437_474);
  });
});
