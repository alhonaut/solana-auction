import * as anchor from "@coral-xyz/anchor";
import { AuctionHouse } from "../target/types/auction_house";
import { Auctioneer } from "../target/types/auctioneer";
import { NftMinter } from "../target/types/nft_minter";

export const AUCTION_HOUSE_PROGRAM_ID = (
  anchor.workspace.AuctionHouse as anchor.Program<AuctionHouse>
).programId;

export const AUCTIONEER_PROGRAM_ID = (
  anchor.workspace.Auctioneer as anchor.Program<Auctioneer>
).programId;

export const NFT_MINTER_PROGRAM_ID = (
  anchor.workspace.NftMinter as anchor.Program<NftMinter>
).programId;

export const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);
