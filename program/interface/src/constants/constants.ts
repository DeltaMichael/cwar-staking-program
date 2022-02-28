import BN from 'bn.js';

/**
 * General constants
 */
export class Constants {
  static stakingTokenDecimals = 9;

  static toCwarRaw = Math.pow(10, Constants.stakingTokenDecimals);

  static maxCwarSupply = new BN(1000_000_000).mul(new BN(Constants.toCwarRaw));

  static rewardTokenDecimals = 9;

  static toRewardTokenRaw = Math.pow(10, Constants.rewardTokenDecimals);

  static cwarPoolBytes = 416;

  static userStorageBytes = 114;

  static u64MaxStrValue = '18446744073709551615';

  static secondsInOneDay = 86400;

  static decimalPrecision = 1000_000;
}
