import {setupTest, timeout} from './testHelpers';
import {TestPool} from './InitializeTestPoolHelper';

setupTest();

describe('Create User Transaction', () => {
  let testPool: TestPool;
  beforeEach(async () => {
    testPool = new TestPool({});
    await testPool.initializePool();
    await testPool.fundPool();
  });

  test('Create User', async () => {
    await testPool.createNewUsers(1);
    const userData = await testPool.getUserData(0);
    expect(userData.accountType).toBe(3);
    expect(userData.getUserWalletPubkey()).toStrictEqual(
      testPool.userWallets[0].publicKey
    );
    expect(userData.getPoolPubkey()).toStrictEqual(
      testPool.cwarPoolStorageAccount.publicKey
    );
    expect(userData.getBalanceStaked()).toBe(0);
    expect(userData.getRewardPerTokenPending()).toBe(0);
    expect(userData.getRewardPerTokenCompleted()).toBe(0);
    const poolData = await testPool.getPoolData();
    expect(userData.getUnstakePenalityDurationEnd()).toBe(
      poolData.getRewardDurationEnd()
    );
    expect(userData.getLastStakedTimestamp()).toBe(0);
  });

  test('Create Same User again throws error', async () => {
    await testPool.createNewUsers();
    await expect(async () => testPool.tryCreateSameUser()).rejects.toThrow();
  });

  test('Create User Again After Closing it', async () => {
    await testPool.createNewUsers(1);
    let userData = await testPool.getUserData(0);
    expect(userData.accountType).toBe(3);
    expect(userData.getUserWalletPubkey()).toStrictEqual(
      testPool.userWallets[0].publicKey
    );
    expect(userData.getPoolPubkey()).toStrictEqual(
      testPool.cwarPoolStorageAccount.publicKey
    );
    expect(userData.getBalanceStaked()).toBe(0);
    expect(userData.getRewardPerTokenPending()).toBe(0);
    expect(userData.getRewardPerTokenCompleted()).toBe(0);
    let poolData = await testPool.getPoolData();
    expect(userData.getUnstakePenalityDurationEnd()).toBe(
      poolData.getRewardDurationEnd()
    );
    expect(userData.getLastStakedTimestamp()).toBe(0);

    await testPool.stakeCwar();
    await timeout(2000);
    await testPool.unstakeCwar();
    await testPool.claimRewards();

    const userWallet = testPool.userWallets[0];

    await testPool.closeUser();

    await testPool.createNewUser(userWallet);
    userData = await testPool.getUserData(0);
    expect(userData.accountType).toBe(3);
    expect(userData.getUserWalletPubkey()).toStrictEqual(
      testPool.userWallets[0].publicKey
    );
    expect(userData.getPoolPubkey()).toStrictEqual(
      testPool.cwarPoolStorageAccount.publicKey
    );
    expect(userData.getBalanceStaked()).toBe(0);
    expect(userData.getRewardPerTokenPending()).toBe(0);
    expect(userData.getRewardPerTokenCompleted()).toBe(0);
    poolData = await testPool.getPoolData();
    expect(userData.getUnstakePenalityDurationEnd()).toBe(
      poolData.getRewardDurationEnd()
    );
    expect(userData.getLastStakedTimestamp()).toBe(0);

    await testPool.stakeCwar();
    await timeout(2000);
    await testPool.unstakeCwar();
    await testPool.claimRewards();
  });
});
