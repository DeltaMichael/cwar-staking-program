import {setupTest} from './testHelpers';
import {TestPool} from './InitializeTestPoolHelper';

setupTest();

describe('Stake Cwar Transaction', () => {
  let testPool: TestPool;

  beforeEach(async () => {
    testPool = new TestPool({});
    await testPool.initializePool();
    await testPool.fundPool();

    await testPool.createNewUsers();
  });

  test('Stake Cwar Tokens', async () => {
    await testPool.stakeCwar();
    let userData = await testPool.getUserData();
    expect(userData.getBalanceStaked()).toBe(100);

    await testPool.stakeCwar();
    userData = await testPool.getUserData();
    expect(userData.getBalanceStaked()).toBe(200);

    await testPool.stakeCwar(1, 20);
    userData = await testPool.getUserData(1);
    expect(userData.getBalanceStaked()).toBe(20);

    const stakingVaultBalanceBefore = await testPool.getStakingVaultBalance();
    await testPool.stakeCwar(4, 90.23);
    userData = await testPool.getUserData(4);
    expect(userData.getBalanceStaked()).toBe(90.23);
    const stakingVaultBalanceAfter = await testPool.getStakingVaultBalance();
    expect(stakingVaultBalanceAfter - stakingVaultBalanceBefore).toBeCloseTo(
      90.23
    );

    userData = await testPool.getUserData(2);
    expect(userData.getBalanceStaked()).toBe(0);

    await testPool.stakeCwar(4, 20);
    userData = await testPool.getUserData(4);
    expect(userData.getBalanceStaked()).toBe(110.23);
  });
});
