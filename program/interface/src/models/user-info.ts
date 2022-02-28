import {Buffer} from 'buffer';
import {PublicKey} from '@solana/web3.js';
import {deserializeUnchecked, serialize} from 'borsh';
import BN from 'bn.js';
import {StringPublicKey} from '../data/ids';
import {ConnectionService} from '../config';
import {extendBorsh} from '../data/borsch';
import {Constants} from '../constants';
import {bigDivWithPrecision} from '../utils';

export class UserData {
  accountType: number;
  userWallet: StringPublicKey;
  cwarPool: StringPublicKey;
  balanceStaked: BN;
  nonce: number;
  rewardPerTokenPending: BN;
  rewardsPerTokenCompleted: BN;
  unstakePenalityDurationEnd: BN;
  lastStakedTimestamp: BN;

  constructor(args: {
    accountType: number;
    userWallet: StringPublicKey;
    cwarPool: StringPublicKey;
    balanceStaked: BN;
    nonce: number;
    rewardPerTokenPending: BN;
    rewardsPerTokenCompleted: BN;
    unstakePenalityDurationEnd: BN;
    lastStakedTimestamp: BN;
  }) {
    this.accountType = args.accountType;
    this.userWallet = args.userWallet;
    this.cwarPool = args.cwarPool;
    this.balanceStaked = args.balanceStaked;
    this.nonce = args.nonce;
    this.rewardPerTokenPending = args.rewardPerTokenPending;
    this.rewardsPerTokenCompleted = args.rewardsPerTokenCompleted;
    this.unstakePenalityDurationEnd = args.unstakePenalityDurationEnd;
    this.lastStakedTimestamp = args.lastStakedTimestamp;
  }

  getUserWalletPubkey(): PublicKey {
    return new PublicKey(this.userWallet);
  }

  getPoolPubkey(): PublicKey {
    return new PublicKey(this.cwarPool);
  }

  getBalanceStaked(): number {
    return bigDivWithPrecision(this.balanceStaked, new BN(Constants.toCwarRaw));
  }

  getNonce(): number {
    return this.nonce;
  }

  getRewardPerTokenPending(): number {
    return bigDivWithPrecision(
      this.rewardPerTokenPending,
      new BN(Constants.toRewardTokenRaw)
    );
  }

  getRewardPerTokenCompleted(): number {
    return bigDivWithPrecision(
      this.rewardsPerTokenCompleted,
      new BN(Constants.toRewardTokenRaw).mul(new BN(Constants.u64MaxStrValue))
    );
  }

  getUnstakePenalityDurationEnd(): number {
    return this.unstakePenalityDurationEnd.toNumber();
  }

  getLastStakedTimestamp(): number {
    return this.lastStakedTimestamp.toNumber();
  }

  printUserData(): void {
    console.log('userWalletPubkey: ', this.getUserWalletPubkey().toString());
    console.log('PoolPubkey: ', this.getPoolPubkey().toString());
    console.log('Balance Staked: ', this.getBalanceStaked());
    console.log('Nonce: ', this.getNonce());
    console.log('rewardPerTokenPending: ', this.getRewardPerTokenPending());
    console.log(
      'rewardsPerTokenCompleted: ',
      this.getRewardPerTokenCompleted()
    );
    console.log(
      'unstakePenalityDurationEnd: ',
      this.getUnstakePenalityDurationEnd()
    );
    console.log('lastStakedTimestamp: ', this.getLastStakedTimestamp());
  }

  static async fromAccount(account: PublicKey): Promise<UserData | null> {
    const connection = ConnectionService.getConnection();
    const accountData = await connection.getAccountInfo(account);
    if (!accountData) return null;
    return UserData.fromBuffer(accountData?.data);
  }

  static fromBuffer(buffer: Buffer): UserData {
    extendBorsh();
    return deserializeUnchecked(
      USER_STORAGE_DATA_ON_CHAIN_SCHEMA,
      UserData,
      buffer.slice(0, USER_STORAGE_TOTAL_BYTES)
    );
  }

  toBuffer(): Buffer {
    extendBorsh();
    return Buffer.from([...serialize(USER_STORAGE_DATA_ON_CHAIN_SCHEMA, this)]);
  }
}

export const USER_STORAGE_TOTAL_BYTES = Constants.userStorageBytes;

export const USER_STORAGE_DATA_ON_CHAIN_SCHEMA = new Map([
  [
    UserData,
    {
      kind: 'struct',
      fields: [
        ['accountType', 'u8'],
        ['userWallet', 'pubkeyAsString'],
        ['cwarPool', 'pubkeyAsString'],
        ['balanceStaked', 'u64'],
        ['nonce', 'u8'],
        ['rewardPerTokenPending', 'u64'],
        ['rewardsPerTokenCompleted', 'u128'],
        ['unstakePenalityDurationEnd', 'u64'],
        ['lastStakedTimestamp', 'u64'],
      ],
    },
  ],
]);
