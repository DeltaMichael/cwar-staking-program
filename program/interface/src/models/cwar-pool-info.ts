import {Buffer} from 'buffer';
import {PublicKey} from '@solana/web3.js';
import {deserializeUnchecked, serialize} from 'borsh';
import BN from 'bn.js';
import {StringPublicKey} from '../data/ids';
import {ConnectionService} from '../config';
import {extendBorsh} from '../data/borsch';
import {Constants} from '../constants';
import {bigDivWithPrecision} from '../utils';

export class CwarPoolData {
  accountType: number;
  ownerWallet: StringPublicKey;
  stakingVault: StringPublicKey;
  stakingMint: StringPublicKey;
  rewardVault: StringPublicKey;
  rewardMint: StringPublicKey;
  rewardRate: BN;
  rewardDuration: BN;
  totalStakeLastUpdateTime: BN;
  rewardPerTokenStored: BN;
  userStakeCount: number;
  pdaNonce: number;
  funders: StringPublicKey[];
  rewardDurationEnd: BN;
  unstakePenalityBasisPoints: number;
  lockingDuration: BN;
  authorityPenalityDepositATA: StringPublicKey;

  constructor(args: {
    accountType: number;
    ownerWallet: StringPublicKey;
    stakingVault: StringPublicKey;
    stakingMint: StringPublicKey;
    rewardVault: StringPublicKey;
    rewardMint: StringPublicKey;
    rewardRate: BN;
    rewardDuration: BN;
    totalStakeLastUpdateTime: BN;
    rewardPerTokenStored: BN;
    userStakeCount: number;
    pdaNonce: number;
    funders: StringPublicKey[];
    rewardDurationEnd: BN;
    unstakePenalityBasisPoints: number;
    lockingDuration: BN;
    authorityPenalityDepositATA: StringPublicKey;
  }) {
    this.accountType = args.accountType;
    this.ownerWallet = args.ownerWallet;
    this.stakingVault = args.stakingVault;
    this.stakingMint = args.stakingMint;
    this.rewardVault = args.rewardVault;
    this.rewardMint = args.rewardMint;
    this.rewardRate = args.rewardRate;
    this.rewardDuration = args.rewardDuration;
    this.totalStakeLastUpdateTime = args.totalStakeLastUpdateTime;
    this.rewardPerTokenStored = args.rewardPerTokenStored;
    this.userStakeCount = args.userStakeCount;
    this.pdaNonce = args.pdaNonce;
    this.funders = args.funders;
    this.rewardDurationEnd = args.rewardDurationEnd;
    this.unstakePenalityBasisPoints = args.unstakePenalityBasisPoints;
    this.lockingDuration = args.lockingDuration;
    this.authorityPenalityDepositATA = args.authorityPenalityDepositATA;
  }

  getAuthorityPubkey(): PublicKey {
    return new PublicKey(this.ownerWallet);
  }

  getStakingVaultPubkey(): PublicKey {
    return new PublicKey(this.stakingVault);
  }

  getStakingMintPubkey(): PublicKey {
    return new PublicKey(this.stakingMint);
  }

  getRewardVaultPubkey(): PublicKey {
    return new PublicKey(this.rewardVault);
  }

  getRewardMintPubkey(): PublicKey {
    return new PublicKey(this.rewardMint);
  }

  getRewardRate(): number {
    return bigDivWithPrecision(this.rewardRate, new BN(Constants.toCwarRaw));
  }

  getRewardDuration(): number {
    return this.rewardDuration.toNumber();
  }

  getRewardDurationInDays(): number {
    return this.rewardDuration.toNumber() / Constants.secondsInOneDay;
  }

  getTotalStakeLastUpdateTime(): number {
    return this.totalStakeLastUpdateTime.toNumber();
  }

  getRewardPerTokenStored(): number {
    return bigDivWithPrecision(
      this.rewardPerTokenStored,
      new BN(Constants.u64MaxStrValue).mul(new BN(Constants.toRewardTokenRaw))
    );
  }

  getUserStakeCount(): number {
    return this.userStakeCount;
  }

  getPdaNonce(): number {
    return this.pdaNonce;
  }

  getFundersArray(): StringPublicKey[] {
    return this.funders;
  }

  getRewardDurationEnd(): number {
    return this.rewardDurationEnd.toNumber();
  }

  getUnstakePenalityBasisPoints(): number {
    return this.unstakePenalityBasisPoints;
  }

  getLockingDuration(): number {
    return this.lockingDuration.toNumber();
  }

  getLockingDurationInDays(): number {
    return this.lockingDuration.toNumber() / Constants.secondsInOneDay;
  }

  getAuthorityPenalityDepositATA(): PublicKey {
    return new PublicKey(this.authorityPenalityDepositATA);
  }

  printPoolInfo(): void {
    console.log('accountType: ', this.accountType);
    console.log('ownerWallet: ', this.getAuthorityPubkey().toString());
    console.log('stakingVault: ', this.getStakingVaultPubkey().toString());
    console.log('stakingMint: ', this.getStakingMintPubkey().toString());
    console.log('rewardVault: ', this.getRewardVaultPubkey().toString());
    console.log('rewardMint: ', this.getRewardMintPubkey().toString());
    console.log('rewardRate: ', this.getRewardRate());
    console.log('rewardDuration: ', this.getRewardDuration());
    console.log(
      'totalStakeLastUpdateTime: ',
      this.getTotalStakeLastUpdateTime()
    );
    console.log('rewardPerTokenStored: ', this.getRewardPerTokenStored());
    console.log('userStakeCount: ', this.getUserStakeCount());
    console.log('pdaNonce: ', this.getPdaNonce());
    console.log('funders: ', this.getFundersArray());
    console.log('rewardDurationEnd: ', this.getRewardDurationEnd());
    console.log(
      'unstakePenalityBasisPoints: ',
      this.getUnstakePenalityBasisPoints()
    );
    console.log('lockingDuration: ', this.getLockingDuration());
    console.log(
      'authorityPenalityDepositATA: ',
      this.getAuthorityPenalityDepositATA().toString()
    );
  }

  static async fromAccount(account: PublicKey): Promise<CwarPoolData | null> {
    const connection = ConnectionService.getConnection();
    const accountData = await connection.getAccountInfo(account);
    if (!accountData) return null;
    return CwarPoolData.fromBuffer(accountData?.data);
  }

  static fromBuffer(buffer: Buffer): CwarPoolData {
    extendBorsh();
    return deserializeUnchecked(
      CWAR_POOL_DATA_ON_CHAIN_SCHEMA,
      CwarPoolData,
      buffer.slice(0, CWAR_POOL_STORAGE_TOTAL_BYTES)
    );
  }

  toBuffer(): Buffer {
    extendBorsh();
    return Buffer.from([...serialize(CWAR_POOL_DATA_ON_CHAIN_SCHEMA, this)]);
  }
}

export const CWAR_POOL_STORAGE_TOTAL_BYTES = Constants.cwarPoolBytes;

export const CWAR_POOL_DATA_ON_CHAIN_SCHEMA = new Map([
  [
    CwarPoolData,
    {
      kind: 'struct',
      fields: [
        ['accountType', 'u8'],
        ['ownerWallet', 'pubkeyAsString'],
        ['stakingVault', 'pubkeyAsString'],
        ['stakingMint', 'pubkeyAsString'],
        ['rewardVault', 'pubkeyAsString'],
        ['rewardMint', 'pubkeyAsString'],
        ['rewardRate', 'u64'],
        ['rewardDuration', 'u64'],
        ['totalStakeLastUpdateTime', 'u64'],
        ['rewardPerTokenStored', 'u128'],
        ['userStakeCount', 'u32'],
        ['pdaNonce', 'u8'],
        ['funders', ['pubkeyAsString', 5]],
        ['rewardDurationEnd', 'u64'],
        ['unstakePenalityBasisPoints', 'u16'],
        ['lockingDuration', 'u64'],
        ['authorityPenalityDepositATA', 'pubkeyAsString'],
      ],
    },
  ],
]);
