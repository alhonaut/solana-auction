import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Auctioneer } from "../../target/types/auctioneer";
import * as pda from "../pda";
import { AuctionHouseData } from "../interfaces";
import { AUCTION_HOUSE_PROGRAM_ID } from "../generated";

export async function withdraw({
  auctioneerProgram,
  auctionHouse,
  buyerKeypair,
  amount,
}: {
  auctioneerProgram: Program<Auctioneer>;
  auctionHouse: AuctionHouseData;
  buyerKeypair: anchor.web3.Keypair;
  amount: number;
}) {
  const [escrowPaymentAccountAddress, escrowBump] =
    pda.findEscrowPaymentAccountAddress({
      wallet: buyerKeypair.publicKey,
      auctionHouseAddress: auctionHouse.address,
    });
  const [auctioneerAuthorityAddress, auctioneerAuthorityBump] =
    pda.findAuctioneerAuthorityAddress({
      auctionHouseAddress: auctionHouse.address,
    });
  const [auctioneerAddress] = pda.findAuctioneerAddress({
    auctionHouseAddress: auctionHouse.address,
    auctioneerAuthorityAddress,
  });

  const withdrawTx = await auctioneerProgram.methods
    .withdraw(escrowBump, auctioneerAuthorityBump, new BN(amount))
    .accounts({
      auctionHouseProgram: AUCTION_HOUSE_PROGRAM_ID,
      wallet: buyerKeypair.publicKey, // Signer
      receiptAccount: buyerKeypair.publicKey,
      escrowPaymentAccount: escrowPaymentAccountAddress,
      treasuryMint: auctionHouse.treasuryMint,
      authority: auctionHouse.authority,
      auctionHouse: auctionHouse.address,
      auctionHouseFeeAccount: auctionHouse.auctionHouseFeeAccount,
      auctioneerAuthority: auctioneerAuthorityAddress,
      auctioneer: auctioneerAddress,
    })
    .signers([buyerKeypair])
    .rpc();
  console.log("Transaction [Withdraw]", withdrawTx);
}
