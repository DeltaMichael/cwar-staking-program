import {setupTest, timeout} from './testHelpers';
import {TestPool} from './InitializeTestPoolHelper';

setupTest();

describe('Locking CWAR Tokens Tests', () => {
  let testPool: TestPool;
  beforeEach(async () => {
    testPool = new TestPool({lockingPeriodDurationDays: 0.0001});
    await testPool.initializePool();
    await testPool.fundPool();

    await testPool.createNewUsers();
  });

  test('Try to Unstake Before Locking Duration is over throws error', async () => {
    await testPool.stakeCwar();

    await expect(async () => testPool.unstakeCwar()).rejects.toThrow();
    await expect(async () => testPool.claimRewards()).rejects.toThrow();
  });
  test('Try to Unstake After Locking Duration is over', async () => {
    await testPool.stakeCwar();
    await timeout(10_000);
    await testPool.unstakeCwar();
    await testPool.claimRewards();
  });
});
