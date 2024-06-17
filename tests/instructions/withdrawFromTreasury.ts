import { Program, BN } from "@coral-xyz/anchor";
import { AuctionHouse } from "../../target/types/auction_house";
import { AuctionHouseData } from "../interfaces";

export async function withdrawFromTreasury({
  auctionHouseProgram,
  auctionHouse,
  amount,
}: {
  auctionHouseProgram: Program<AuctionHouse>;
  auctionHouse: AuctionHouseData;
  amount: number;
}) {
  const withdrawFromFeeTx = await auctionHouseProgram.methods
    .withdrawFromTreasury(new BN(amount))
    .accounts({
      treasuryMint: auctionHouse.treasuryMint,
      authority: auctionHouse.authorityKeypair.publicKey,
      treasuryWithdrawalDestination: auctionHouse.treasuryWithdrawalDestination,
      auctionHouseTreasury: auctionHouse.auctionHouseTreasury,
      auctionHouse: auctionHouse.address,
    })
    .signers([auctionHouse.authorityKeypair])
    .rpc();
  console.log("Transaction [Withdraw From Treasury]", withdrawFromFeeTx);
}
