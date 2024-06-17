import { Program, BN } from "@coral-xyz/anchor";
import { AuctionHouse } from "../../target/types/auction_house";
import { AuctionHouseData } from "../interfaces";

export async function withdrawFromFee({
  auctionHouseProgram,
  auctionHouse,
  amount,
}: {
  auctionHouseProgram: Program<AuctionHouse>;
  auctionHouse: AuctionHouseData;
  amount: number;
}) {
  const withdrawFromFeeTx = await auctionHouseProgram.methods
    .withdrawFromFee(new BN(amount))
    .accounts({
      authority: auctionHouse.authorityKeypair.publicKey,
      feeWithdrawalDestination: auctionHouse.feeWithdrawalDestination,
      auctionHouseFeeAccount: auctionHouse.auctionHouseFeeAccount,
      auctionHouse: auctionHouse.address,
    })
    .signers([auctionHouse.authorityKeypair])
    .rpc();
  console.log("Transaction [Withdraw From Fee]", withdrawFromFeeTx);
}
