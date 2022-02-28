import {getTokenBalance, setupTest, timeout} from './testHelpers';
import {TestPool} from './InitializeTestPoolHelper';

setupTest();

describe('Unstake Penality CWAR Token', () => {
  let testPool: TestPool;

  beforeEach(async () => {
    testPool = new TestPool({
      lockingPeriodDurationDays: 0,
      unstakePenalityPercentage: 5,
    });
    await testPool.initializePool();
    await testPool.fundPool(0.0002);

    await testPool.createNewUsers();
  });

  test('Unstake Before Pool Ends', async () => {
    await testPool.stakeCwar();
    const userStakingAtaBalanceBefore =
      await testPool.getUserStakingAtaBalance();
    const adminUnstakePenalityWalletBalanceBefore = await getTokenBalance(
      testPool.authorityPenalityDepositAta
    );
    await testPool.unstakeCwar();
    const userStakingAtaBalanceAfter =
      await testPool.getUserStakingAtaBalance();
    expect(userStakingAtaBalanceAfter - userStakingAtaBalanceBefore).toBe(
      0.95 * 100
    );
    const adminUnstakePenalityWalletBalanceAfter = await getTokenBalance(
      testPool.authorityPenalityDepositAta
    );
    expect(
      adminUnstakePenalityWalletBalanceAfter -
        adminUnstakePenalityWalletBalanceBefore
    ).toBe(5);
  });

  test('Unstake After Pool Ends', async () => {
    await testPool.stakeCwar();
    const userStakingAtaBalanceBefore =
      await testPool.getUserStakingAtaBalance();
    await timeout(10_000);
    const adminUnstakePenalityWalletBalanceBefore = await getTokenBalance(
      testPool.authorityPenalityDepositAta
    );
    await testPool.unstakeCwar();
    const userStakingAtaBalanceAfter =
      await testPool.getUserStakingAtaBalance();
    expect(userStakingAtaBalanceAfter - userStakingAtaBalanceBefore).toBe(100);
    const adminUnstakePenalityWalletBalanceAfter = await getTokenBalance(
      testPool.authorityPenalityDepositAta
    );
    expect(
      adminUnstakePenalityWalletBalanceAfter -
        adminUnstakePenalityWalletBalanceBefore
    ).toBe(0);
  });
});
