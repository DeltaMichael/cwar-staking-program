import {setupTest, timeout} from './testHelpers';
import {TestPool} from './InitializeTestPoolHelper';
import {getUserPendingRewards} from '../src/models';
import BN from 'bn.js';

setupTest();

describe('Unstake Cwar Transaction', () => {
  let testPool: TestPool;

  beforeEach(async () => {
    testPool = new TestPool({});
    await testPool.initializePool();
    await testPool.fundPool();

    await testPool.createNewUsers();
  });

  test('Unstake Cwar Tokens', async () => {
    await testPool.stakeCwar();
    let userData = await testPool.getUserData();
    expect(userData.getBalanceStaked()).toBe(100);

    await testPool.stakeCwar();
    userData = await testPool.getUserData();
    expect(userData.getBalanceStaked()).toBe(200);

    await testPool.unstakeCwar(0, 50);
    userData = await testPool.getUserData();
    expect(userData.getBalanceStaked()).toBe(150);

    await testPool.stakeCwar(1, 20);
    userData = await testPool.getUserData(1);
    expect(userData.getBalanceStaked()).toBe(20);

    await testPool.unstakeCwar(1, 10);
    userData = await testPool.getUserData(1);
    expect(userData.getBalanceStaked()).toBe(10);

    let stakingVaultBalanceBefore = await testPool.getStakingVaultBalance();
    await testPool.stakeCwar(4, 90.23);
    userData = await testPool.getUserData(4);
    expect(userData.getBalanceStaked()).toBe(90.23);
    let stakingVaultBalanceAfter = await testPool.getStakingVaultBalance();
    expect(stakingVaultBalanceAfter - stakingVaultBalanceBefore).toBeCloseTo(
      90.23
    );

    userData = await testPool.getUserData(2);
    expect(userData.getBalanceStaked()).toBe(0);

    await testPool.stakeCwar(4, 20);
    //const poolDataBefore = await testPool.getPoolData();
    userData = await testPool.getUserData(4);
    expect(userData.getBalanceStaked()).toBe(110.23);

    const userRewardAtaBalanceBefore = await testPool.getUserRewardAtaBalance(
      4
    );
    const userStakeAtaBalanceBefore = await testPool.getUserStakingAtaBalance(
      4
    );
    userData = await testPool.getUserData(4);
    stakingVaultBalanceBefore = await testPool.getStakingVaultBalance();
    await testPool.unstakeCwar(4, 40.12);
    //const poolDataAfter = await testPool.getPoolData();

    const userStakeAtaBalanceAfter = await testPool.getUserStakingAtaBalance(4);
    const userRewardAtaBalanceAfter = await testPool.getUserRewardAtaBalance(4);
    expect(userStakeAtaBalanceAfter - userStakeAtaBalanceBefore).toBeCloseTo(
      40.12
    );

    // let userPendingRewards = await getUserPendingRewards(
    //   testPool.userWallets[4].publicKey,
    //   poolDataAfter.getTotalStakeLastUpdateTime(),
    //   poolDataBefore.getTotalStakeLastUpdateTime()
    // );
    // userPendingRewards = new BN(userPendingRewards).toNumber() / 1000_000_000;
    // expect(userRewardAtaBalanceAfter - userRewardAtaBalanceBefore).toBe(
    //   userPendingRewards
    // );

    console.log(
      'userRewardAtaBalanceAfter - userRewardAtaBalanceBefore: ',
      userRewardAtaBalanceAfter - userRewardAtaBalanceBefore
    );
    console.log(
      'getUserPendingRewards(testPool.userWallets[4].publicKey): ',
      await getUserPendingRewards(testPool.userWallets[4].publicKey)
    );
    userData = await testPool.getUserData(4);
    expect(userData.getBalanceStaked()).toBe(70.11);
    stakingVaultBalanceAfter = await testPool.getStakingVaultBalance();
    expect(stakingVaultBalanceBefore - stakingVaultBalanceAfter).toBeCloseTo(
      40.12
    );
  });

  test('Unstake Cwar Tokens test pending rewards', async () => {
    const userIndex = 4;
    await testPool.stakeCwar(userIndex);
    const userRewardAtaBalanceBefore = await testPool.getUserRewardAtaBalance(
      userIndex
    );
    const poolDataBefore = await testPool.getPoolData();
    await timeout(9_000);
    await testPool.unstakeCwar(userIndex, 40.12);

    const userRewardAtaBalanceAfter = await testPool.getUserRewardAtaBalance(
      userIndex
    );
    const poolDataAfter = await testPool.getPoolData();

    let userPendingRewards = await getUserPendingRewards(
      testPool.userWallets[userIndex].publicKey,
      poolDataAfter.getTotalStakeLastUpdateTime(),
      poolDataBefore.getTotalStakeLastUpdateTime()
    );
    userPendingRewards = new BN(userPendingRewards).toNumber() / 1000_000_000;
    expect(userRewardAtaBalanceAfter - userRewardAtaBalanceBefore).toBe(
      userPendingRewards
    );
  });
});
