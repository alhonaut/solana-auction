import { Program, Wallet, Provider } from "@coral-xyz/anchor";

import { AuctionHouse } from "../../target/types/auction_house";
import { Auctioneer } from "../../target/types/auctioneer";
import { NATIVE_MINT } from "@solana/spl-token";

import * as pda from "../pda";
import { createSystemAccount } from "../utils";
import { AuctionHouseData } from "../interfaces";

export async function createAuctionHouse({
  auctionHouseProgram,
  auctioneerProgram,
  provider,
  payer,
}: {
  auctionHouseProgram: Program<AuctionHouse>;
  auctioneerProgram: Program<Auctioneer>;
  provider: Provider;
  payer: Wallet;
}) {
  const authorityKeypair = await createSystemAccount({ provider, payer });

  const treasuryMint = NATIVE_MINT;

  const [auctionHouseAddress, auctionHouseBump] = pda.findAuctionHouseAddress({
    authority: authorityKeypair.publicKey,
    mint: treasuryMint,
  });
  const [auctionHouseFeeAccountAddress, feePayerBump] =
    pda.findAuctionHouseFeeAccountAddress({ auctionHouseAddress });
  const [auctionHouseTreasuryAddress, treasuryBump] =
    pda.findAuctionHouseTreasuryAddress({ auctionHouseAddress });
  const sellerFeeBasisPoints = 100;
  const canChangeSalePrice = false;

  const createAuctionHouseTx = await auctionHouseProgram.methods
    .createAuctionHouse(
      auctionHouseBump,
      feePayerBump,
      treasuryBump,
      sellerFeeBasisPoints,
      canChangeSalePrice
    )
    .accounts({
      treasuryMint,
      payer: authorityKeypair.publicKey, // Signer
      authority: authorityKeypair.publicKey,
      feeWithdrawalDestination: payer.publicKey,
      treasuryWithdrawalDestination: payer.publicKey,
      treasuryWithdrawalDestinationOwner: payer.publicKey,
      auctionHouse: auctionHouseAddress,
      auctionHouseFeeAccount: auctionHouseFeeAccountAddress,
      auctionHouseTreasury: auctionHouseTreasuryAddress,
    })
    .signers([authorityKeypair])
    .rpc();
  console.log("Transaction [Create Auction House]", createAuctionHouseTx);

  const [auctioneerAuthorityAddress] = pda.findAuctioneerAuthorityAddress({
    auctionHouseAddress,
  });
  const [auctioneerAddress] = pda.findAuctioneerAddress({
    auctionHouseAddress,
    auctioneerAuthorityAddress,
  });

  const delegateAuctioneerTx = await auctionHouseProgram.methods
    .delegateAuctioneer()
    .accounts({
      auctionHouse: auctionHouseAddress,
      authority: authorityKeypair.publicKey, // Signer
      auctioneerAuthority: auctioneerAuthorityAddress,
      auctioneer: auctioneerAddress,
    })
    .signers([authorityKeypair])
    .rpc();
  console.log("Transaction [Delegate Auctioneer]", delegateAuctioneerTx);

  const authorizeAuctioneerTx = await auctioneerProgram.methods
    .authorize()
    .accounts({
      wallet: authorityKeypair.publicKey, // Signer
      auctionHouse: auctionHouseAddress,
      auctioneerAuthority: auctioneerAuthorityAddress,
    })
    .signers([authorityKeypair])
    .rpc();
  console.log("Transaction [Authorize Auctioneer]", authorizeAuctioneerTx);

  const ah = await auctionHouseProgram.account.auctionHouse.fetch(
    auctionHouseAddress
  );

  const auctionHouse: AuctionHouseData = {
    address: auctionHouseAddress,
    authorityKeypair: authorityKeypair,
    auctionHouseFeeAccount: ah.auctionHouseFeeAccount,
    auctionHouseTreasury: ah.auctionHouseTreasury,
    treasuryWithdrawalDestination: ah.treasuryWithdrawalDestination,
    feeWithdrawalDestination: ah.feeWithdrawalDestination,
    treasuryMint: ah.treasuryMint,
    authority: ah.authority,
    creator: ah.creator,
    bump: ah.bump,
    treasuryBump: ah.treasuryBump,
    feePayerBump: ah.feePayerBump,
    sellerFeeBasisPoints: ah.sellerFeeBasisPoints,
    canChangeSalePrice: ah.canChangeSalePrice,
    escrowPaymentBump: ah.escrowPaymentBump,
    hasAuctioneer: ah.hasAuctioneer,
    auctioneerAddress: ah.auctioneerAddress,
  };

  return auctionHouse;
}
