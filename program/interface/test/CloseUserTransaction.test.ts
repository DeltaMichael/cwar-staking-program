import {setupTest, timeout} from './testHelpers';
import {TestPool} from './InitializeTestPoolHelper';
import {getUserStorageAccount} from '../src/utils';

setupTest();

describe('Close User Transaction', () => {
  let testPool: TestPool;
  beforeEach(async () => {
    testPool = new TestPool({});
    await testPool.initializePool();
    await testPool.fundPool();

    await testPool.createNewUsers();
  });

  test('Close User', async () => {
    await testPool.stakeCwar();

    await testPool.unstakeCwar();

    await testPool.claimRewards();

    const userStoragePubkey = await getUserStorageAccount(
      testPool.userWallets[0].publicKey
    );
    const txFee = 5000;
    const userWalletBalanceBefore = await testPool.connection.getBalance(
      testPool.userWallets[0].publicKey
    );
    const userStorageBalanceBefore = await testPool.connection.getBalance(
      userStoragePubkey
    );
    await testPool.closeUser();
    const userStorageBalanceAfter = await testPool.connection.getBalance(
      userStoragePubkey
    );
    const userWalletBalanceAfter = await testPool.connection.getBalance(
      testPool.userWallets[0].publicKey
    );
    expect(userStorageBalanceAfter).toBe(0);
    expect(userWalletBalanceAfter).toBe(
      userWalletBalanceBefore + userStorageBalanceBefore - txFee
    );
  });

  test('Close User with non zero staking balance throws error', async () => {
    await testPool.stakeCwar();

    await testPool.claimRewards();

    await testPool.unstakeCwar(0, 50);

    await expect(async () => testPool.closeUser()).rejects.toThrow();
  });

  test('Close User with non-zero unclaimed rewards throws error', async () => {
    await testPool.stakeCwar();

    await timeout(2000);

    await expect(async () => testPool.closeUser()).rejects.toThrow();
  });

  test('Close User with someone else wallet throws error', async () => {
    await testPool.stakeCwar();

    await testPool.claimRewards();

    await testPool.unstakeCwar();

    await expect(async () =>
      testPool.closeUserWithRandomSigner()
    ).rejects.toThrow();
  });
});
