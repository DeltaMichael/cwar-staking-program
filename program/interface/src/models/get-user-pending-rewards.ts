import {PublicKey} from '@solana/web3.js';
import BN from 'bn.js';
import {CwarPoolData, UserData} from '.';
import {Constants, Pubkeys} from '../constants';
import {getStakingVaultBalanceRaw} from '../data';
import {getUserStorageAccount} from '../utils';

export async function getUserPendingRewards(
  userWallet: PublicKey,
  lastApplicableTime?: number,
  lastTotalStakeUpdateTime?: number
): Promise<number> {
  const U64_MAX = new BN(Constants.u64MaxStrValue, 10);
  const cwarPoolData = await CwarPoolData.fromAccount(
    Pubkeys.cwarPoolStoragePubkey
  );
  if (cwarPoolData === null) {
    throw new Error('Pool Does Not Exist');
  }
  const userDataStorageAddress = await getUserStorageAccount(userWallet);
  const userData = await UserData.fromAccount(userDataStorageAddress);
  if (userData === null) {
    return 0;
  }
  const totalTokensStakedRaw = await getStakingVaultBalanceRaw();
  if (!lastApplicableTime)
    lastApplicableTime = Math.min(
      Math.floor(Date.now() / 1000),
      cwarPoolData.rewardDurationEnd.toNumber()
    );
  if (!lastTotalStakeUpdateTime)
    lastTotalStakeUpdateTime = cwarPoolData.totalStakeLastUpdateTime.toNumber();
  const timeElasped = new BN(lastApplicableTime - lastTotalStakeUpdateTime);
  let currentRewardPerToken = cwarPoolData.rewardPerTokenStored;
  if (totalTokensStakedRaw > new BN(0)) {
    currentRewardPerToken = cwarPoolData.rewardPerTokenStored.add(
      timeElasped
        .mul(cwarPoolData.rewardRate)
        .mul(U64_MAX)
        .div(totalTokensStakedRaw)
    );
  }

  const userPendingRewards = userData.balanceStaked
<<<<<<< HEAD
    .mul(currentRewardPerToken.sub(userData.rewardsPerTokenCompleted))
=======
    .mul(
      currentRewardPerToken
        .sub(userData.rewardsPerTokenCompleted)
    )
>>>>>>> 1456d98 (Get pending rewards working)
    .div(U64_MAX)
    .add(userData.rewardPerTokenPending)
    .toNumber();
  return userPendingRewards;
}
