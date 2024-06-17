import { Program, BN } from "@coral-xyz/anchor";
import { Auctioneer } from "../../target/types/auctioneer";
import { AuctionHouseData, NFT, SellAccounts } from "../interfaces";
import * as pda from "../pda";
import { AUCTION_HOUSE_PROGRAM_ID } from "../generated";

export async function sell({
  auctioneerProgram,
  auctionHouse,
  token,
  startTime,
  endTime,
  reservePrice,
  minBidIncrement,
  timeExtPeriod,
  timeExtDelta,
  tokenSize,
}: {
  auctioneerProgram: Program<Auctioneer>;
  auctionHouse: AuctionHouseData;
  token: NFT;
  startTime: number;
  endTime: number;
  reservePrice?: number;
  minBidIncrement?: number;
  timeExtPeriod?: number;
  timeExtDelta?: number;
  tokenSize: number;
}) {
  const [sellerTradeStateAddress, sellerTradeStateBump] =
    pda.findAuctioneerTradeStateAddress({
      wallet: token.owner.publicKey,
      auctionHouseAddress: auctionHouse.address,
      tokenAccount: token.ata,
      treasuryMint: auctionHouse.treasuryMint,
      tokenMint: token.mint.publicKey,
      tokenSize,
    });

  const [freeSellerTradeStateAddress, freeSellerTradeStateBump] =
    pda.findTradeStateAddress({
      wallet: token.owner.publicKey,
      auctionHouseAddress: auctionHouse.address,
      tokenAccount: token.ata,
      treasuryMint: auctionHouse.treasuryMint,
      tokenMint: token.mint.publicKey,
      price: 0,
      tokenSize,
    });

  const [listingConfigAddress] = pda.findListingConfigAddress({
    wallet: token.owner.publicKey,
    auctionHouseAddress: auctionHouse.address,
    tokenAccount: token.ata,
    treasuryMint: auctionHouse.treasuryMint,
    tokenMint: token.mint.publicKey,
    tokenSize,
  });

  const [programAsSignerAddress, programAsSignerBump] =
    pda.findProgramAsSignerAddress();
  const [auctioneerAuthorityAddress, auctioneerAuthorityBump] =
    pda.findAuctioneerAuthorityAddress({
      auctionHouseAddress: auctionHouse.address,
    });
  const [auctioneerAddress] = pda.findAuctioneerAddress({
    auctionHouseAddress: auctionHouse.address,
    auctioneerAuthorityAddress,
  });

  const sellAccounts: SellAccounts = {
    auctionHouseProgram: AUCTION_HOUSE_PROGRAM_ID,
    listingConfig: listingConfigAddress,
    wallet: token.owner.publicKey, // Signer
    tokenAccount: token.ata,
    metadata: token.metadata,
    authority: auctionHouse.authority,
    auctionHouse: auctionHouse.address,
    auctionHouseFeeAccount: auctionHouse.auctionHouseFeeAccount,
    sellerTradeState: sellerTradeStateAddress,
    freeSellerTradeState: freeSellerTradeStateAddress,
    programAsSigner: programAsSignerAddress,
    auctioneerAuthority: auctioneerAuthorityAddress,
    auctioneer: auctioneerAddress,
  };

  const sellTx = await auctioneerProgram.methods
    .sell(
      sellerTradeStateBump,
      freeSellerTradeStateBump,
      programAsSignerBump,
      auctioneerAuthorityBump,
      new BN(tokenSize),
      new BN(startTime),
      new BN(endTime),
      new BN(reservePrice ?? 0),
      new BN(minBidIncrement ?? 0),
      timeExtPeriod ?? 0,
      timeExtDelta ?? 0
    )
    .accounts(sellAccounts)
    .signers([token.owner])
    .rpc();
  console.log("Transaction [Sell]", sellTx);

  return { sellAccounts };
}
