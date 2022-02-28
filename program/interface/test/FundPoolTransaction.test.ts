import {
  getAdminKeypair,
  requestAirdrop,
  setupTest,
  timeout,
} from './testHelpers';
import {Keypair} from '@solana/web3.js';
import {Constants} from '../src/constants';
import {TestPool} from './InitializeTestPoolHelper';
setupTest();

describe('Fund Pool Transaction', () => {
  let testPool: TestPool;
  const adminKeypair: Keypair = getAdminKeypair();
  beforeEach(async () => {
    testPool = new TestPool({});
    await testPool.initializePool();
  });

  test('Does not change pool end date on subsequent funding', async () => {
    await testPool.fundPool(1, adminKeypair);
    await timeout(2000);
    const poolDataAfter = await testPool.getPoolData();
    const poolRewardDurationEnd = poolDataAfter.getRewardDurationEnd();
    expect(poolDataAfter.rewardRate.toNumber()).toBe(1 * Constants.toCwarRaw);
    expect(await testPool.getRewardsVaultBalance()).toBe(86_400);
    console.log('Sleeping for 2 seconds...');
    await timeout(2000);
    await testPool.fundPool(0, adminKeypair);
    await timeout(3000);
    const poolDataAfterAfter = await testPool.getPoolData();
    const poolRewardDurationEndAfter =
      poolDataAfterAfter.getRewardDurationEnd();
    expect(poolRewardDurationEndAfter).toBe(poolRewardDurationEnd);
  });

  test('Try funding pool with wrong Funder', async () => {
    const funderWallet = Keypair.generate();
    await requestAirdrop(funderWallet.publicKey);
    //const poolDataBefore = await testPool.getPoolData();
    //expect(poolDataBefore.rewardRate).toBe(0);
    await expect(async () =>
      testPool.fundPool(1, funderWallet)
    ).rejects.toThrow();
    const poolDataAfter = await testPool.getPoolData();
    expect(poolDataAfter.rewardRate).toBe(0);
    expect(await testPool.getRewardsVaultBalance()).toBe(0);
  });

  test('Fund pool with admin wallet', async () => {
    const poolDataBefore = await testPool.getPoolData();
    expect(poolDataBefore.rewardRate).toBe(0);
    await testPool.fundPool(1, adminKeypair);
    const poolDataAfter = await testPool.getPoolData();
    expect(poolDataAfter.rewardRate).toBe(1 * Constants.toCwarRaw);
    expect(await testPool.getRewardsVaultBalance()).toBe(86_400);
  });

  test('Fund pool with funder wallet', async () => {
    const funderWallet = Keypair.generate();
    await testPool.addFunder(funderWallet.publicKey, adminKeypair);
    const poolDataBefore = await testPool.getPoolData();
    expect(poolDataBefore.rewardRate).toBe(0);
    await testPool.fundPool(1, funderWallet);
    const poolDataAfter = await testPool.getPoolData();
    expect(poolDataAfter.rewardRate).toBe(1 * Constants.toCwarRaw);
    expect(await testPool.getRewardsVaultBalance()).toBe(86_400);
  });

  test('Fund pool with 2nd funder wallet', async () => {
    const funderWallet1 = Keypair.generate();
    await testPool.addFunder(funderWallet1.publicKey, adminKeypair);
    const funderWallet2 = Keypair.generate();
    await testPool.addFunder(funderWallet2.publicKey, adminKeypair);
    //const poolDataBefore = await testPool.getPoolData();
    //expect(poolDataBefore.rewardRate).toBe(0);
    await testPool.fundPool(1, funderWallet2);
    const poolDataAfter = await testPool.getPoolData();
    expect(poolDataAfter.rewardRate).toBe(1 * Constants.toCwarRaw);
    expect(await testPool.getRewardsVaultBalance()).toBe(86_400);
  });
});
