import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Auctioneer } from "../../target/types/auctioneer";
import * as pda from "../pda";
import { NFT, AuctionHouseData, BuyAccounts } from "../interfaces";
import { AUCTION_HOUSE_PROGRAM_ID } from "../generated";

export async function buy({
  auctioneerProgram,
  auctionHouse,
  token,
  buyerKeypair,
  sellerAddress,
  buyerPrice,
  tokenSize,
}: {
  auctioneerProgram: Program<Auctioneer>;
  auctionHouse: AuctionHouseData;
  token: NFT;
  buyerKeypair: anchor.web3.Keypair;
  sellerAddress: anchor.web3.PublicKey;
  buyerPrice: number;
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

  const [buyerTradeStateAddress, buyerTradeStateBump] =
    pda.findTradeStateAddress({
      wallet: buyerKeypair.publicKey,
      auctionHouseAddress: auctionHouse.address,
      tokenAccount: token.ata, // seller token account
      treasuryMint: auctionHouse.treasuryMint,
      tokenMint: token.mint.publicKey,
      price: buyerPrice,
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

  const buyAccounts: BuyAccounts = {
    auctionHouseProgram: AUCTION_HOUSE_PROGRAM_ID,
    listingConfig: listingConfigAddress,
    seller: sellerAddress,
    wallet: buyerKeypair.publicKey, // Signer
    paymentAccount: buyerKeypair.publicKey,
    transferAuthority: buyerKeypair.publicKey,
    treasuryMint: auctionHouse.treasuryMint,
    tokenAccount: token.ata, // seller token account
    metadata: token.metadata,
    escrowPaymentAccount: escrowPaymentAccountAddress,
    authority: auctionHouse.authority,
    auctionHouse: auctionHouse.address,
    auctionHouseFeeAccount: auctionHouse.auctionHouseFeeAccount,
    buyerTradeState: buyerTradeStateAddress,
    auctioneerAuthority: auctioneerAuthorityAddress,
    auctioneer: auctioneerAddress,
  };

  const buyTx = await auctioneerProgram.methods
    .buy(
      buyerTradeStateBump,
      escrowBump,
      auctioneerAuthorityBump,
      new BN(buyerPrice),
      new BN(tokenSize)
    )
    .accounts(buyAccounts)
    .signers([buyerKeypair])
    .rpc();
  console.log("Transaction [Buy]", buyTx);

  return { buyAccounts };
}
