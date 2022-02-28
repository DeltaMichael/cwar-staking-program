import {getAdminKeypair, requestAirdrop, setupTest} from './testHelpers';
import {Keypair, PublicKey} from '@solana/web3.js';
import {TestPool} from './InitializeTestPoolHelper';

setupTest();

describe('Add Funder Transaction', () => {
  let testPool: TestPool;
  const adminKeypair: Keypair = getAdminKeypair();
  beforeEach(async () => {
    testPool = new TestPool({});
    await testPool.initializePool();
  });

  test('Add 1 Funder', async () => {
    const funderWallet = Keypair.generate();
    await testPool.addFunder(funderWallet.publicKey, adminKeypair);
    await testPool.fundPool(1, funderWallet);
    expect(await testPool.getRewardsVaultBalance()).toBe(86_400);
    expect(await testPool.getStakingVaultBalance()).toBe(0);
  });

  test('Try adding same Funder again throws error', async () => {
    const funderWallet = Keypair.generate();
    await testPool.addFunder(funderWallet.publicKey, adminKeypair);
    const poolData = await testPool.getPoolData();
    expect(poolData.funders[0]).toBe(funderWallet.publicKey.toString());
    await expect(async () =>
      testPool.addFunder(funderWallet.publicKey, adminKeypair)
    ).rejects.toThrow();
    await testPool.fundPool(1, funderWallet);
    expect(await testPool.getRewardsVaultBalance()).toBe(86_400);
    expect(await testPool.getStakingVaultBalance()).toBe(0);
  });

  test('Add 2 Funders', async () => {
    const funderWallets = [Keypair.generate(), Keypair.generate()];
    await testPool.addFunder(funderWallets[0].publicKey, adminKeypair);
    await testPool.fundPool(1, funderWallets[0]);
    expect(await testPool.getRewardsVaultBalance()).toBe(86_400);
    const poolData = await testPool.getPoolData();
    expect(poolData.funders[0]).toBe(funderWallets[0].publicKey.toString());
    // adding same funder again throws error
    await expect(async () =>
      testPool.addFunder(funderWallets[0].publicKey, adminKeypair)
    ).rejects.toThrow();

    await testPool.addFunder(funderWallets[1].publicKey, adminKeypair);
    const poolDataAfter = await testPool.getPoolData();
    expect(poolDataAfter.funders[1]).toBe(
      funderWallets[1].publicKey.toString()
    );

    // adding same funder again throws error
    await expect(async () =>
      testPool.addFunder(funderWallets[1].publicKey, adminKeypair)
    ).rejects.toThrow();
    await testPool.fundPool(1, funderWallets[1]);
    expect(await testPool.getRewardsVaultBalance()).toBe(2 * 86_400);
    expect(await testPool.getStakingVaultBalance()).toBe(0);
  });

  test('Try adding Funder with wrong authority throws error', async () => {
    const funderWallet = Keypair.generate();
    const admin = Keypair.generate();
    await requestAirdrop(admin.publicKey);
    const poolData = await testPool.getPoolData();
    expect(poolData.funders[0]).toBe(PublicKey.default.toString());
    await expect(async () =>
      testPool.addFunder(funderWallet.publicKey, admin)
    ).rejects.toThrow();
    expect(poolData.funders[0]).toBe(PublicKey.default.toString());
    await expect(async () =>
      testPool.fundPool(1, funderWallet)
    ).rejects.toThrow();
    expect(await testPool.getRewardsVaultBalance()).toBe(0);
    expect(await testPool.getStakingVaultBalance()).toBe(0);
  });

  test('Add all 5 funders', async () => {
    const funderWallets: Keypair[] = [];

    for (let i = 0; i < 5; i++) {
      const funderWallet = Keypair.generate();
      funderWallets.push(funderWallet);
      await testPool.addFunder(funderWallet.publicKey, adminKeypair);
      const poolData = await testPool.getPoolData();
      expect(poolData.funders[i]).toBe(funderWallet.publicKey.toString());
    }

    for (let i = 0; i < 5; i++) {
      await testPool.fundPool(1, funderWallets[i]);
      expect(await testPool.getRewardsVaultBalance()).toBe((i + 1) * 86_400);
      const poolData = await testPool.getPoolData();
      expect(poolData.rewardRate.toNumber()).toBeCloseTo(1_000_000_000);
    }

    //adding 6th funder throws error
    const funderWallet = Keypair.generate();
    await expect(async () =>
      testPool.addFunder(funderWallet.publicKey, adminKeypair)
    ).rejects.toThrow();
    expect(await testPool.getRewardsVaultBalance()).toBe(5 * 86_400);
    expect(await testPool.getStakingVaultBalance()).toBe(0);
  });
});
