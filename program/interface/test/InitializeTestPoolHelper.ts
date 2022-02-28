import {Token, TOKEN_PROGRAM_ID} from '@solana/spl-token';
import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import {ConnectionService} from '../src/config';
import {Constants, Pubkeys} from '../src/constants';
import {CwarPoolData, UserData} from '../src/models';
import {
  addFunderTransaction,
  claimRewardsTransaction,
  closePoolTransaction,
  closeUserTransaction,
  createInitializePoolTransaction,
  createUserTransaction,
  fundPoolTransaction,
  removeFunderTransaction,
  stakeCwarTransaction,
  unstakeCwarTransaction,
} from '../src/transactions';
import {findAssociatedTokenAddress, getUserStorageAccount} from '../src/utils';
import {
  findAndCreateAta,
  findAndCreateRewardAta,
  findAndCreateStakingAta,
  getAdminKeypair,
  getTokenBalance,
  mintTokensTo,
  requestAirdrop,
} from './testHelpers';

export class TestPool {
  cwarPoolStorageAccount = Keypair.generate();
  cwarStakingVault = Keypair.generate();
  cwarRewardsVault = Keypair.generate();
  stakingTokenMint: Token;
  rewardTokenMint: Token;
  adminKeypair: Keypair;
  lockingPeriodDurationDays: number;
  unstakePenalityPercentage: number;
  stakingTokenDecimals: number;
  rewardTokenDecimals: number;
  userWallets: Keypair[] = [];
  connection: Connection;
  funders: Keypair[] = [];
  authorityPenalityDepositAta: PublicKey;
  /**
   * By Default lockingPeriodDurationDays = 0, unstakePenalityPercentage = 0,
   * stakingTokenDecimals = 9, rewardTokenDecimals = 9
   */
  constructor(args: {
    lockingPeriodDurationDays?: number;
    unstakePenalityPercentage?: number;
    stakingTokenDecimals?: number;
    rewardTokenDecimals?: number;
  }) {
    this.lockingPeriodDurationDays = args.lockingPeriodDurationDays ?? 0;
    this.unstakePenalityPercentage = args.unstakePenalityPercentage ?? 0;
    this.stakingTokenDecimals = args.stakingTokenDecimals ?? 9;
    this.rewardTokenDecimals = args.rewardTokenDecimals ?? 9;
    this.adminKeypair = getAdminKeypair();
    this.connection = ConnectionService.getConnection();
    this.stakingTokenMint = new Token(
      this.connection,
      this.adminKeypair.publicKey,
      TOKEN_PROGRAM_ID,
      this.adminKeypair
    );
    this.rewardTokenMint = new Token(
      this.connection,
      this.adminKeypair.publicKey,
      TOKEN_PROGRAM_ID,
      this.adminKeypair
    );
    this.authorityPenalityDepositAta = PublicKey.default;
  }
  /**
   * Initializes Test Pool
   */
  async initializePool(): Promise<void> {
    Constants.stakingTokenDecimals = this.stakingTokenDecimals;
    Constants.rewardTokenDecimals = this.rewardTokenDecimals;
    await requestAirdrop(this.adminKeypair.publicKey);
    // Create Cwar Test Token
    const stakingTokenMint = await Token.createMint(
      this.connection,
      this.adminKeypair,
      this.adminKeypair.publicKey,
      null,
      this.stakingTokenDecimals,
      TOKEN_PROGRAM_ID
    );
    Pubkeys.stakingMintPubkey = stakingTokenMint.publicKey;
    this.stakingTokenMint = stakingTokenMint;

    // Create Reward Test Token
    const rewardTokenMint = await Token.createMint(
      this.connection,
      this.adminKeypair,
      this.adminKeypair.publicKey,
      null,
      this.rewardTokenDecimals,
      TOKEN_PROGRAM_ID
    );
    Pubkeys.rewardsMintPubkey = rewardTokenMint.publicKey;
    this.rewardTokenMint = rewardTokenMint;

    const initializePoolTx = await createInitializePoolTransaction(
      this.adminKeypair.publicKey,
      this.cwarPoolStorageAccount,
      this.cwarStakingVault,
      this.cwarRewardsVault,
      this.unstakePenalityPercentage,
      this.lockingPeriodDurationDays
    );
    await sendAndConfirmTransaction(this.connection, initializePoolTx, [
      this.adminKeypair,
      this.cwarPoolStorageAccount,
      this.cwarStakingVault,
      this.cwarRewardsVault,
    ]);
    Pubkeys.cwarPoolStoragePubkey = this.cwarPoolStorageAccount.publicKey;
    Pubkeys.cwarStakingVaultPubkey = this.cwarStakingVault.publicKey;
    Pubkeys.cwarRewardsVaultPubkey = this.cwarRewardsVault.publicKey;
    Pubkeys.unstakePenalityATAPubkey = await findAndCreateAta(
      this.adminKeypair,
      Pubkeys.stakingMintPubkey
    );
    this.authorityPenalityDepositAta = await findAssociatedTokenAddress(
      this.adminKeypair.publicKey,
      Pubkeys.stakingMintPubkey
    );
  }

  /**
   * By Default extensionInDays = 1, rewardTokenAmountToFundPoolWith = 86_400,
   * funderWallet = adminKeypair
   */
  async fundPool(
    extensionInDays?: number,
    funderWallet?: Keypair,
    rewardTokenAmountToFundPoolWith?: number
  ): Promise<void> {
    if (!funderWallet) funderWallet = this.adminKeypair;
    if (!rewardTokenAmountToFundPoolWith)
      rewardTokenAmountToFundPoolWith = 86_400;
    // Mint 86_400 Reward token by default to funder wallet to fund pool for testing
    await requestAirdrop(funderWallet.publicKey);
    await mintTokensTo(
      this.rewardTokenMint,
      funderWallet,
      this.adminKeypair.publicKey,
      rewardTokenAmountToFundPoolWith
    );

    if (extensionInDays === undefined) extensionInDays = 1;
    const fundPoolTx = await fundPoolTransaction(
      funderWallet.publicKey,
      rewardTokenAmountToFundPoolWith,
      extensionInDays
    );
    await sendAndConfirmTransaction(this.connection, fundPoolTx, [
      funderWallet,
    ]);
  }

  /**
   * By Default numUsers = 10 and amountStakingTokensToMint = 1_000
   * This function
   * 1. Airdrop SOL to new Users
   * 2. Create Staking ATAs and Mint Staking Tokens for testing pool
   * 3. Create User Data Accounts on blockchain
   */
  async createNewUsers(
    numUsers?: number,
    amountStakingTokensToMint?: number
  ): Promise<void> {
    if (!numUsers) numUsers = 10;
    for (let i = 0; i < numUsers; i++) {
      this.userWallets.push(Keypair.generate());
    }

    // Airdrop SOL to all user wallets
    await Promise.all(
      this.userWallets.map(userWallet => requestAirdrop(userWallet.publicKey))
    );

    if (!amountStakingTokensToMint) amountStakingTokensToMint = 1_000;
    // Mint 1000 Staking token by default to all user wallet ATAs for testing
    await Promise.all(
      this.userWallets.map(userWallet =>
        mintTokensTo(
          this.stakingTokenMint,
          userWallet,
          this.adminKeypair.publicKey,
          amountStakingTokensToMint as number
        )
      )
    );
    //Create User data accounts on blockchain
    for (let i = 0; i < this.userWallets.length; i++) {
      const createUserTx = await createUserTransaction(
        this.userWallets[i].publicKey
      );
      await sendAndConfirmTransaction(this.connection, createUserTx, [
        this.userWallets[i],
      ]);
    }
  }

  /**
   * By Default userIndex = 0, amount = 100
   */
  async stakeCwar(userIndex?: number, amount?: number): Promise<void> {
    if (!userIndex) userIndex = 0;
    if (!amount) amount = 100;
    const stakeCwarTx = await stakeCwarTransaction(
      this.userWallets[userIndex].publicKey,
      amount
    );
    await sendAndConfirmTransaction(this.connection, stakeCwarTx, [
      this.userWallets[userIndex],
    ]);
  }

  /**
   * By Default userIndex = 0, amount = 100
   */
  async unstakeCwar(userIndex?: number, amount?: number): Promise<void> {
    if (!userIndex) userIndex = 0;
    if (!amount) amount = 100;
    const unstakeCwarTx = await unstakeCwarTransaction(
      this.userWallets[userIndex].publicKey,
      amount
    );
    await sendAndConfirmTransaction(this.connection, unstakeCwarTx, [
      this.userWallets[userIndex],
    ]);
  }

  /**
   * By Default userIndex = 0
   */
  async claimRewards(userIndex?: number): Promise<void> {
    if (!userIndex) userIndex = 0;
    const claimRewardsTx = await claimRewardsTransaction(
      this.userWallets[userIndex].publicKey
    );
    await sendAndConfirmTransaction(this.connection, claimRewardsTx, [
      this.userWallets[userIndex],
    ]);
  }

  /**
   * By Default userIndex = 0
   */
  async closeUser(userIndex?: number): Promise<void> {
    if (!userIndex) userIndex = 0;
    const closeUserTx = await closeUserTransaction(
      this.userWallets[userIndex].publicKey
    );
    await sendAndConfirmTransaction(this.connection, closeUserTx, [
      this.userWallets[userIndex],
    ]);
    this.userWallets.splice(userIndex, userIndex);
  }

  /**
   * By Default userIndex = 0
   */
  async closeUserWithRandomSigner(userIndex?: number): Promise<void> {
    if (!userIndex) userIndex = 0;
    const randomWallet = Keypair.generate();
    const closeUserTx = await closeUserTransaction(
      this.userWallets[userIndex].publicKey
    );
    await sendAndConfirmTransaction(this.connection, closeUserTx, [
      randomWallet,
    ]);
    this.userWallets.splice(userIndex, userIndex);
  }

  async closeAllUsers(): Promise<void> {
    for (let i = this.userWallets.length - 1; i >= 0; i--) {
      await this.closeUser(i);
    }
  }

  async closePool(admin?: Keypair): Promise<void> {
    if (!admin) admin = this.adminKeypair;
    const closePoolTx = await closePoolTransaction(admin.publicKey);
    await sendAndConfirmTransaction(this.connection, closePoolTx, [admin]);
  }

  async addFunder(funderWallet: PublicKey, authority: Keypair): Promise<void> {
    const addFunderTx = await addFunderTransaction(
      authority.publicKey,
      funderWallet
    );
    await requestAirdrop(funderWallet);
    await requestAirdrop(authority.publicKey);
    await sendAndConfirmTransaction(this.connection, addFunderTx, [authority]);
  }

  async removeFunder(
    funderWallet: PublicKey,
    authority: Keypair
  ): Promise<void> {
    const removeFunderTx = await removeFunderTransaction(
      authority.publicKey,
      funderWallet
    );
    await sendAndConfirmTransaction(this.connection, removeFunderTx, [
      authority,
    ]);
  }

  async getPoolData(): Promise<CwarPoolData> {
    const poolData = await CwarPoolData.fromAccount(
      this.cwarPoolStorageAccount.publicKey
    );
    return poolData!;
  }

  async getUserData(userIndex?: number): Promise<UserData> {
    if (!userIndex) userIndex = 0;
    const userStoragePubkey = await getUserStorageAccount(
      this.userWallets[userIndex].publicKey
    );
    const userData = await UserData.fromAccount(userStoragePubkey);
    return userData!;
  }

  async getStakingVaultBalance(): Promise<number> {
    const tokenBalance = await this.connection.getTokenAccountBalance(
      this.cwarStakingVault.publicKey
    );
    return tokenBalance.value.uiAmount!;
  }

  async getRewardsVaultBalance(): Promise<number> {
    const tokenBalance = await this.connection.getTokenAccountBalance(
      this.cwarRewardsVault.publicKey
    );
    return tokenBalance.value.uiAmount!;
  }

  async tryCreateSameUser(userIndex?: number): Promise<void> {
    if (!userIndex) userIndex = 0;

    const createUserTx = await createUserTransaction(
      this.userWallets[userIndex].publicKey
    );
    await sendAndConfirmTransaction(this.connection, createUserTx, [
      this.userWallets[userIndex],
    ]);
  }

  async createNewUser(
    userWallet: Keypair,
    amountStakingTokensToMint?: number
  ): Promise<void> {
    await requestAirdrop(userWallet.publicKey);
    this.userWallets.push(userWallet);

    if (!amountStakingTokensToMint) amountStakingTokensToMint = 1_000;
    // Mint 1000 Staking token by default to all user wallet ATAs for testing

    await mintTokensTo(
      this.stakingTokenMint,
      userWallet,
      this.adminKeypair.publicKey,
      amountStakingTokensToMint as number
    );

    const createUserTx = await createUserTransaction(userWallet.publicKey);
    await sendAndConfirmTransaction(this.connection, createUserTx, [
      userWallet,
    ]);
  }

  async getUserStakingAtaBalance(userIndex?: number): Promise<number> {
    if (!userIndex) userIndex = 0;
    const userStakingAta = await findAndCreateStakingAta(
      this.userWallets[userIndex]
    );
    const tokenBalance = await getTokenBalance(userStakingAta);
    return tokenBalance;
  }

  async getUserRewardAtaBalance(userIndex?: number): Promise<number> {
    if (!userIndex) userIndex = 0;
    const userRewardAta = await findAndCreateRewardAta(
      this.userWallets[userIndex]
    );
    const tokenBalance = await getTokenBalance(userRewardAta);
    return tokenBalance;
  }
}
