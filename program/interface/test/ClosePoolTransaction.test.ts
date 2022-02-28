import {
  findAndCreateRewardAta,
  findAndCreateStakingAta,
  getAdminKeypair,
  getSolBalance,
  getTokenBalance,
  setupTest,
  timeout,
} from './testHelpers';
import {Keypair} from '@solana/web3.js';
import {TestPool} from './InitializeTestPoolHelper';

setupTest();

describe('Close Pool Transaction', () => {
  let testPool: TestPool;
  const adminKeypair: Keypair = getAdminKeypair();
  beforeEach(async () => {
    testPool = new TestPool({});
    await testPool.initializePool();
    await testPool.fundPool(0.0002);

    const funderWallet = Keypair.generate();
    await testPool.addFunder(funderWallet.publicKey, adminKeypair);
    await testPool.removeFunder(funderWallet.publicKey, adminKeypair);
  });

  test('Close Pool', async () => {
    await testPool.createNewUsers();
    await testPool.stakeCwar();

    await testPool.claimRewards();

    await testPool.unstakeCwar();

    await testPool.closeAllUsers();
    const adminRewardAta = await findAndCreateRewardAta(testPool.adminKeypair);
    const adminStakeAta = await findAndCreateStakingAta(testPool.adminKeypair);
    const txFee = 5000;
    const adminWalletBalanceBefore = await getSolBalance(
      testPool.adminKeypair.publicKey
    );
    const adminRewardAtaBalanceBefore = await getTokenBalance(adminRewardAta);
    const adminStakingAtaBalanceBefore = await getTokenBalance(adminStakeAta);
    const stakingVaultBalanceBefore = await testPool.getStakingVaultBalance();
    const rewardsVaultBalanceBefore = await testPool.getRewardsVaultBalance();
    const stakingVaultSolBalanceBefore = await getSolBalance(
      testPool.cwarStakingVault.publicKey
    );
    const rewardsVaultSolBalanceBefore = await getSolBalance(
      testPool.cwarRewardsVault.publicKey
    );
    await timeout(9000);
    await testPool.closePool();

    const adminRewardAtaBalanceAfter = await getTokenBalance(adminRewardAta);
    const adminStakingAtaBalanceAfter = await getTokenBalance(adminStakeAta);
    const stakingVaultSolBalanceAfter = await getSolBalance(
      testPool.cwarStakingVault.publicKey
    );
    const rewardsVaultSolBalanceAfter = await getSolBalance(
      testPool.cwarRewardsVault.publicKey
    );

    const adminWalletBalanceAfter = await getSolBalance(
      testPool.adminKeypair.publicKey
    );
    expect(rewardsVaultSolBalanceAfter).toBe(0);
    expect(stakingVaultSolBalanceAfter).toBe(0);
    expect(adminRewardAtaBalanceBefore + rewardsVaultBalanceBefore).toBe(
      adminRewardAtaBalanceAfter
    );
    expect(adminStakingAtaBalanceBefore + stakingVaultBalanceBefore).toBe(
      adminStakingAtaBalanceAfter
    );
    expect(adminWalletBalanceAfter).toBe(
      adminWalletBalanceBefore +
        rewardsVaultSolBalanceBefore +
        stakingVaultSolBalanceBefore -
        txFee
    );
  });

  test('Close Pool before reward duration end throws error', async () => {
    await testPool.createNewUsers();
    await testPool.stakeCwar();

    await testPool.claimRewards();

    await testPool.unstakeCwar();

    await testPool.closeAllUsers();
    await expect(async () => testPool.closePool()).rejects.toThrow();
  });

  test('Close Pool with non zero users throws error', async () => {
    await testPool.createNewUsers();
    await testPool.stakeCwar();

    await testPool.claimRewards();

    await testPool.unstakeCwar();

    await testPool.closeAllUsers();

    await testPool.createNewUsers(1);
    await expect(async () => testPool.closePool()).rejects.toThrow();
  });

  test('Close Pool with fake owner throws error', async () => {
    await testPool.createNewUsers();
    await testPool.stakeCwar();

    await testPool.claimRewards();

    await testPool.unstakeCwar();

    await testPool.closeAllUsers();

    const fakeAdmin = Keypair.generate();
    await expect(async () => testPool.closePool(fakeAdmin)).rejects.toThrow();
  });
});
