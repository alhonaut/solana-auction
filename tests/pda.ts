import * as anchor from "@coral-xyz/anchor";
import {
  AUCTION_HOUSE_PROGRAM_ID,
  AUCTIONEER_PROGRAM_ID,
  TOKEN_METADATA_PROGRAM_ID,
} from "./generated";
import { MAX_UINT64 } from "./utils";

const METADATA_PREFIX = "metadata";
const EDITION = "edition";

const PREFIX = "auction_house";
const FEE_PAYER = "fee_payer";
const TREASURY = "treasury";
const AUCTIONEER = "auctioneer";
const LISTING_CONFIG = "listing_config";
const SIGNER = "signer";

export const findMetadataAddress = ({
  mint,
}: {
  mint: anchor.web3.PublicKey;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(METADATA_PREFIX),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );

export const findEditionAddress = ({
  mint,
}: {
  mint: anchor.web3.PublicKey;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(METADATA_PREFIX),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from(EDITION),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );

export const findAuctionHouseAddress = ({
  authority,
  mint,
}: {
  authority: anchor.web3.PublicKey;
  mint: anchor.web3.PublicKey;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(PREFIX), authority.toBuffer(), mint.toBuffer()],
    AUCTION_HOUSE_PROGRAM_ID
  );

export const findAuctionHouseFeeAccountAddress = ({
  auctionHouseAddress,
}: {
  auctionHouseAddress: anchor.web3.PublicKey;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(PREFIX),
      auctionHouseAddress.toBuffer(),
      Buffer.from(FEE_PAYER),
    ],
    AUCTION_HOUSE_PROGRAM_ID
  );

export const findAuctionHouseTreasuryAddress = ({
  auctionHouseAddress,
}: {
  auctionHouseAddress: anchor.web3.PublicKey;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(PREFIX),
      auctionHouseAddress.toBuffer(),
      Buffer.from(TREASURY),
    ],
    AUCTION_HOUSE_PROGRAM_ID
  );

export const findAuctioneerAuthorityAddress = ({
  auctionHouseAddress,
}: {
  auctionHouseAddress: anchor.web3.PublicKey;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(AUCTIONEER), auctionHouseAddress.toBuffer()],
    AUCTIONEER_PROGRAM_ID
  );

export const findAuctioneerAddress = ({
  auctionHouseAddress,
  auctioneerAuthorityAddress,
}: {
  auctionHouseAddress: anchor.web3.PublicKey;
  auctioneerAuthorityAddress: anchor.web3.PublicKey;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(AUCTIONEER),
      auctionHouseAddress.toBuffer(),
      auctioneerAuthorityAddress.toBuffer(),
    ],
    AUCTION_HOUSE_PROGRAM_ID
  );

export const findAuctioneerTradeStateAddress = ({
  wallet,
  auctionHouseAddress,
  tokenAccount,
  treasuryMint,
  tokenMint,
  tokenSize,
}: {
  wallet: anchor.web3.PublicKey;
  auctionHouseAddress: anchor.web3.PublicKey;
  tokenAccount: anchor.web3.PublicKey;
  treasuryMint: anchor.web3.PublicKey;
  tokenMint: anchor.web3.PublicKey;
  tokenSize: number;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(PREFIX),
      wallet.toBuffer(),
      auctionHouseAddress.toBuffer(),
      tokenAccount.toBuffer(),
      treasuryMint.toBuffer(),
      tokenMint.toBuffer(),
      new anchor.BN(MAX_UINT64).toBuffer("le", 8),
      new anchor.BN(tokenSize.toString()).toBuffer("le", 8),
    ],
    AUCTION_HOUSE_PROGRAM_ID
  );

export const findTradeStateAddress = ({
  wallet,
  auctionHouseAddress,
  tokenAccount,
  treasuryMint,
  tokenMint,
  price,
  tokenSize,
}: {
  wallet: anchor.web3.PublicKey;
  auctionHouseAddress: anchor.web3.PublicKey;
  tokenAccount: anchor.web3.PublicKey;
  treasuryMint: anchor.web3.PublicKey;
  tokenMint: anchor.web3.PublicKey;
  price: number;
  tokenSize: number;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(PREFIX),
      wallet.toBuffer(),
      auctionHouseAddress.toBuffer(),
      tokenAccount.toBuffer(),
      treasuryMint.toBuffer(),
      tokenMint.toBuffer(),
      new anchor.BN(price.toString()).toBuffer("le", 8),
      new anchor.BN(tokenSize.toString()).toBuffer("le", 8),
    ],
    AUCTION_HOUSE_PROGRAM_ID
  );

export const findListingConfigAddress = ({
  wallet,
  auctionHouseAddress,
  tokenAccount,
  treasuryMint,
  tokenMint,
  tokenSize,
}: {
  wallet: anchor.web3.PublicKey;
  auctionHouseAddress: anchor.web3.PublicKey;
  tokenAccount: anchor.web3.PublicKey;
  treasuryMint: anchor.web3.PublicKey;
  tokenMint: anchor.web3.PublicKey;
  tokenSize: number;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(LISTING_CONFIG),
      wallet.toBuffer(),
      auctionHouseAddress.toBuffer(),
      tokenAccount.toBuffer(),
      treasuryMint.toBuffer(),
      tokenMint.toBuffer(),
      new anchor.BN(tokenSize.toString()).toBuffer("le", 8),
    ],
    AUCTIONEER_PROGRAM_ID
  );

export const findProgramAsSignerAddress = (): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(PREFIX), Buffer.from(SIGNER)],
    AUCTION_HOUSE_PROGRAM_ID
  );

export const findEscrowPaymentAccountAddress = ({
  wallet,
  auctionHouseAddress,
}: {
  wallet: anchor.web3.PublicKey;
  auctionHouseAddress: anchor.web3.PublicKey;
}): [anchor.web3.PublicKey, number] =>
  anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(PREFIX), auctionHouseAddress.toBuffer(), wallet.toBuffer()],
    AUCTION_HOUSE_PROGRAM_ID
  );
