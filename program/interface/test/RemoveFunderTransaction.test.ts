import {getAdminKeypair, requestAirdrop, setupTest} from './testHelpers';
import {Keypair, PublicKey} from '@solana/web3.js';
import {TestPool} from './InitializeTestPoolHelper';

setupTest();

describe('Remove Funder Transaction', () => {
  let testPool: TestPool;
  const adminKeypair: Keypair = getAdminKeypair();
  beforeEach(async () => {
    testPool = new TestPool({});
    await testPool.initializePool();
  });

  test('Remove Funder', async () => {
    const funderWallet = Keypair.generate();
    await testPool.addFunder(funderWallet.publicKey, adminKeypair);
    await testPool.removeFunder(funderWallet.publicKey, adminKeypair);
  });

  test('Remove all 5 funders', async () => {
    const funderWallets: Keypair[] = [];

    for (let i = 0; i < 5; i++) {
      const funderWallet = Keypair.generate();
      funderWallets.push(funderWallet);
      await testPool.addFunder(funderWallet.publicKey, adminKeypair);
    }

    const shuffledFunderWallets: Keypair[] = funderWallets.sort((v1, v2) => {
      if (v1.publicKey.toString() > v2.publicKey.toString()) {
        return 1;
      }

      if (v1.publicKey.toString() < v2.publicKey.toString()) {
        return -1;
      }

      return 0;
    });

    for (let i = 0; i < 5; i++) {
      await testPool.removeFunder(
        shuffledFunderWallets[i].publicKey,
        adminKeypair
      );
    }
    const poolData = await testPool.getPoolData();
    for (let i = 0; i < 5; i++) {
      expect(poolData.funders[i]).toBe(PublicKey.default.toString());
    }
  });
  test('Try removing admin wallet throws error', async () => {
    await expect(async () =>
      testPool.removeFunder(adminKeypair.publicKey, adminKeypair)
    ).rejects.toThrow();
  });

  test('Try removing funder with wrong admin wallet throws error', async () => {
    const funderWallet = Keypair.generate();
    await testPool.addFunder(funderWallet.publicKey, adminKeypair);
    const admin = Keypair.generate();
    await requestAirdrop(admin.publicKey);
    await expect(async () =>
      testPool.removeFunder(funderWallet.publicKey, admin)
    ).rejects.toThrow();
  });

  test('Try removing funder that is not present throws error', async () => {
    const funderWallet = Keypair.generate();
    await expect(async () =>
      testPool.removeFunder(funderWallet.publicKey, adminKeypair)
    ).rejects.toThrow();
  });
});
