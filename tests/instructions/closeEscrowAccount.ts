import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AuctionHouse } from "../../target/types/auction_house";
import * as pda from "../pda";
import { AuctionHouseData } from "../interfaces";

export async function closeEscrowAccount({
  auctionHouseProgram,
  auctionHouse,
  walletKeypair,
}: {
  auctionHouseProgram: Program<AuctionHouse>;
  auctionHouse: AuctionHouseData;
  walletKeypair: anchor.web3.Keypair;
}) {
  const [escrowPaymentAccountAddress, escrowBump] =
    pda.findEscrowPaymentAccountAddress({
      wallet: walletKeypair.publicKey,
      auctionHouseAddress: auctionHouse.address,
    });

  const closeEscrowAccountTx = await auctionHouseProgram.methods
    .closeEscrowAccount(escrowBump)
    .accounts({
      wallet: walletKeypair.publicKey,
      escrowPaymentAccount: escrowPaymentAccountAddress,
      auctionHouse: auctionHouse.address,
    })
    .signers([walletKeypair])
    .rpc();
  console.log("Transaction [Close Escrow Account]", closeEscrowAccountTx);
}
