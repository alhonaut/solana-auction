import * as anchor from "@coral-xyz/anchor";

export interface AuctionHouseData {
  // Additional Info
  address: anchor.web3.PublicKey;
  authorityKeypair: anchor.web3.Keypair;

  // Account State
  auctionHouseFeeAccount: anchor.web3.PublicKey;
  auctionHouseTreasury: anchor.web3.PublicKey;
  treasuryWithdrawalDestination: anchor.web3.PublicKey;
  feeWithdrawalDestination: anchor.web3.PublicKey;
  treasuryMint: anchor.web3.PublicKey;
  authority: anchor.web3.PublicKey;
  creator: anchor.web3.PublicKey;
  bump: number;
  treasuryBump: number;
  feePayerBump: number;
  sellerFeeBasisPoints: number;
  canChangeSalePrice: boolean;
  escrowPaymentBump: number;
  hasAuctioneer: boolean;
  auctioneerAddress: anchor.web3.PublicKey;
}

export interface NFT {
  mint: anchor.web3.Keypair;
  owner: anchor.web3.Keypair;
  ata: anchor.web3.PublicKey;
  metadata: anchor.web3.PublicKey;
  masterEdition: anchor.web3.PublicKey;
}

export interface SellAccounts {
  auctionHouseProgram: anchor.web3.PublicKey;
  listingConfig: anchor.web3.PublicKey;
  wallet: anchor.web3.PublicKey;
  tokenAccount: anchor.web3.PublicKey;
  metadata: anchor.web3.PublicKey;
  authority: anchor.web3.PublicKey;
  auctionHouse: anchor.web3.PublicKey;
  auctionHouseFeeAccount: anchor.web3.PublicKey;
  sellerTradeState: anchor.web3.PublicKey;
  freeSellerTradeState: anchor.web3.PublicKey;
  programAsSigner: anchor.web3.PublicKey;
  auctioneerAuthority: anchor.web3.PublicKey;
  auctioneer: anchor.web3.PublicKey;
}

export interface BuyAccounts {
  auctionHouseProgram: anchor.web3.PublicKey;
  listingConfig: anchor.web3.PublicKey;
  seller: anchor.web3.PublicKey;
  wallet: anchor.web3.PublicKey;
  paymentAccount: anchor.web3.PublicKey;
  transferAuthority: anchor.web3.PublicKey;
  treasuryMint: anchor.web3.PublicKey;
  tokenAccount: anchor.web3.PublicKey;
  metadata: anchor.web3.PublicKey;
  escrowPaymentAccount: anchor.web3.PublicKey;
  authority: anchor.web3.PublicKey;
  auctionHouse: anchor.web3.PublicKey;
  auctionHouseFeeAccount: anchor.web3.PublicKey;
  buyerTradeState: anchor.web3.PublicKey;
  auctioneerAuthority: anchor.web3.PublicKey;
  auctioneer: anchor.web3.PublicKey;
}

export interface Creator {
  address: anchor.web3.PublicKey;
  share: number;
  verified: boolean;
}
