import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Auctioneer } from "../../target/types/auctioneer";
import * as pda from "../pda";
import { NFT, AuctionHouseData } from "../interfaces";
import { AUCTION_HOUSE_PROGRAM_ID } from "../generated";

export async function deposit({
  auctioneerProgram,
  auctionHouse,
  token,
  buyerKeypair,
  amount,
  tokenSize,
}: {
  auctioneerProgram: Program<Auctioneer>;
  auctionHouse: AuctionHouseData;
  token: NFT;
  buyerKeypair: anchor.web3.Keypair;
  amount: number;
  tokenSize: number;
}) {
  const [buyerTradeStateAddress, buyerTradeStateBump] =
    pda.findTradeStateAddress({
      wallet: buyerKeypair.publicKey,
      auctionHouseAddress: auctionHouse.address,
      tokenAccount: token.ata, // seller token account
      treasuryMint: auctionHouse.treasuryMint,
      tokenMint: token.mint.publicKey,
      price: amount,
      tokenSize,
    });

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

  const depositTx = await auctioneerProgram.methods
    .deposit(escrowBump, auctioneerAuthorityBump, new BN(amount))
    .accounts({
      auctionHouseProgram: AUCTION_HOUSE_PROGRAM_ID,
      wallet: buyerKeypair.publicKey, // Signer
      paymentAccount: buyerKeypair.publicKey,
      transferAuthority: buyerKeypair.publicKey,
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
  console.log("Transaction [Deposit]", depositTx);
}
