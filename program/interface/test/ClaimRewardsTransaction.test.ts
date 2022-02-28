import {setupTest, timeout} from './testHelpers';
import {TestPool} from './InitializeTestPoolHelper';

setupTest();

describe('Claim Rewards Transaction', () => {
  let testPool: TestPool;

  beforeEach(async () => {
    testPool = new TestPool({});
    await testPool.initializePool();
    await testPool.fundPool();
  });

  test('Claim Rewards with 1 user in pool', async () => {
    await testPool.createNewUsers(1);
    await testPool.stakeCwar();
    await testPool.unstakeCwar();
    await testPool.claimRewards();

    await testPool.stakeCwar();
    await timeout(2_000);
    await testPool.claimRewards();
    await testPool.unstakeCwar();

    //const user1RewardTokenBalanceAfter
  });

  test('Claim Rewards with 2 users in pool', async () => {
    await timeout(7_000);
    await testPool.createNewUsers(2);
    const pool_data = await testPool.getPoolData();
    const user1RewardAtaBalanceBefore = await testPool.getUserRewardAtaBalance(
      0
    );
    const user2RewardAtaBalanceBefore = await testPool.getUserRewardAtaBalance(
      1
    );
    // stake 200 tokens from user1
    await testPool.stakeCwar(0, 200);
    const lastUpdateTimestampBefore = (
      await testPool.getPoolData()
    ).getTotalStakeLastUpdateTime();
    await timeout(2_000);
    // stake 100 tokens from user2
    await testPool.stakeCwar(1, 100);
    await timeout(5_000);

    // Unstake 100 Tokens User1
    await testPool.unstakeCwar(0, 100);
    await timeout(2_000);
    // Unstake 100 Tokens User2
    await testPool.unstakeCwar(1, 100);
    await timeout(4_000);
    // Unstake 100 Tokens User1
    await testPool.unstakeCwar(0, 100);
    const lastUpdateTimestampAfter = (
      await testPool.getPoolData()
    ).getTotalStakeLastUpdateTime();
    const user1RewardAtaBalanceAfter = await testPool.getUserRewardAtaBalance(
      0
    );
    const user2RewardAtaBalanceAfter = await testPool.getUserRewardAtaBalance(
      1
    );
    const timeLength = lastUpdateTimestampAfter - lastUpdateTimestampBefore;
    const rewardsShouldBeDistributed = timeLength * pool_data.getRewardRate();
    expect(rewardsShouldBeDistributed).toBeCloseTo(
      user1RewardAtaBalanceAfter -
        user1RewardAtaBalanceBefore +
        user2RewardAtaBalanceAfter -
        user2RewardAtaBalanceBefore
    );
    await timeout(1_000);
    // const user1RewardTokenBalanceBefore
    await testPool.claimRewards(0);
    await testPool.claimRewards(1);

    //const user1RewardTokenBalanceAfter
  });

  test('Claim Rewards with 2 users in pool with multiple stake and unstake', async () => {
    await testPool.createNewUsers(2);
    await timeout(7_000);
    // stake 100 tokens from user1
    await testPool.stakeCwar(0, 100);
    await timeout(2_000);
    // stake 100 tokens from user2
    await testPool.stakeCwar(1, 100);
    await timeout(3_000);

    // stake 100 tokens from user1
    await testPool.stakeCwar(0, 100);
    await timeout(2_000);

    // Unstake 200 Tokens User1
    await testPool.unstakeCwar(0, 200);

    await timeout(4_000);
    // Unstake 100 Tokens User2
    await testPool.unstakeCwar(1, 100);

    await timeout(1_000);
    // const user1RewardTokenBalanceBefore
    await testPool.claimRewards(0);
    await testPool.claimRewards(1);

    //const user1RewardTokenBalanceAfter

    //const user2RewardTokenBalanceAfter
  });
});
