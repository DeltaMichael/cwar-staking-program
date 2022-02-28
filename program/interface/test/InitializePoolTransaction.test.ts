import {setupTest} from './testHelpers';
import {TestPool} from './InitializeTestPoolHelper';

setupTest();

describe('Initialize Cwar Pool Transaction', () => {
  let testPool: TestPool;
  beforeEach(async () => {
    testPool = new TestPool({});
  });

  test('Initialize Pool', async () => {
    await testPool.initializePool();
    expect(await testPool.getRewardsVaultBalance()).toBe(0);
    expect(await testPool.getStakingVaultBalance()).toBe(0);
    const poolData = await testPool.getPoolData();
    expect(poolData.accountType).toBe(2);
    expect(poolData.ownerWallet).toBe(
      testPool.adminKeypair.publicKey.toString()
    );
    expect(poolData.stakingVault).toBe(
      testPool.cwarStakingVault.publicKey.toString()
    );
    expect(poolData.stakingMint).toBe(
      testPool.stakingTokenMint.publicKey.toString()
    );
    expect(poolData.rewardVault).toBe(
      testPool.cwarRewardsVault.publicKey.toString()
    );
    expect(poolData.rewardMint).toBe(
      testPool.rewardTokenMint.publicKey.toString()
    );
    expect(poolData.rewardRate.toNumber()).toBe(0);
    expect(poolData.rewardDuration.toNumber()).toBe(0);
    expect(poolData.totalStakeLastUpdateTime.toNumber()).toBe(0);
    expect(poolData.rewardPerTokenStored.toNumber()).toBe(0);
    expect(poolData.userStakeCount).toBe(0);
    expect(poolData.rewardDurationEnd.toNumber()).toBe(0);
    expect(poolData.unstakePenalityBasisPoints).toBe(0);
    expect(poolData.lockingDuration.toNumber()).toBe(0);
    expect(poolData.authorityPenalityDepositATA).toBe(
      testPool.authorityPenalityDepositAta.toString()
    );
  });
});
