import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Auctioneer } from "../../target/types/auctioneer";
import * as pda from "../pda";
import { NFT, AuctionHouseData } from "../interfaces";
import { AUCTION_HOUSE_PROGRAM_ID } from "../generated";

export async function cancel({
  auctioneerProgram,
  auctionHouse,
  token,
  walletKeypair,
  tradeStateAddress,
  buyerPrice,
  tokenSize,
}: {
  auctioneerProgram: Program<Auctioneer>;
  auctionHouse: AuctionHouseData;
  token: NFT;
  walletKeypair: anchor.web3.Keypair;
  tradeStateAddress: anchor.web3.PublicKey;
  buyerPrice: BN;
  tokenSize: number;
}) {
  const [listingConfigAddress] = pda.findListingConfigAddress({
    wallet: token.owner.publicKey,
    auctionHouseAddress: auctionHouse.address,
    tokenAccount: token.ata,
    treasuryMint: auctionHouse.treasuryMint,
    tokenMint: token.mint.publicKey,
    tokenSize,
  });
  const [auctioneerAuthorityAddress, auctioneerAuthorityBump] =
    pda.findAuctioneerAuthorityAddress({
      auctionHouseAddress: auctionHouse.address,
    });
  const [auctioneerAddress] = pda.findAuctioneerAddress({
    auctionHouseAddress: auctionHouse.address,
    auctioneerAuthorityAddress,
  });

  const cancelTx = await auctioneerProgram.methods
    .cancel(auctioneerAuthorityBump, new BN(buyerPrice), new BN(tokenSize))
    .accounts({
      auctionHouseProgram: AUCTION_HOUSE_PROGRAM_ID,
      listingConfig: listingConfigAddress,
      seller: token.owner.publicKey,
      wallet: walletKeypair.publicKey,
      tokenAccount: token.ata,
      tokenMint: token.mint.publicKey,
      authority: auctionHouse.authority,
      auctionHouse: auctionHouse.address,
      auctionHouseFeeAccount: auctionHouse.auctionHouseFeeAccount,
      tradeState: tradeStateAddress,
      auctioneerAuthority: auctioneerAuthorityAddress,
      auctioneer: auctioneerAddress,
    })
    .signers([walletKeypair])
    .rpc();
  console.log("Transaction [Cancel]", cancelTx);
}
