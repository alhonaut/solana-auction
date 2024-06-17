import * as anchor from "@coral-xyz/anchor";
import { Program, BN, Wallet } from "@coral-xyz/anchor";
import { AuctionHouse } from "../target/types/auction_house";
import { Auctioneer } from "../target/types/auctioneer";
import { NftMinter } from "../target/types/nft_minter";

import { createAuctionHouse } from "./instructions/createAuctionHouse";
import { sell } from "./instructions/sell";
import { buy } from "./instructions/buy";
import { deposit } from "./instructions/deposit";
import { executeSale } from "./instructions/executeSale";
import { cancel } from "./instructions/cancel";
import { withdraw } from "./instructions/withdraw";
import { createNft } from "./instructions/createNft";
import { closeEscrowAccount } from "./instructions/closeEscrowAccount";
import { withdrawFromFee } from "./instructions/withdrawFromFee";
import { withdrawFromTreasury } from "./instructions/withdrawFromTreasury";

import { sleep, createSystemAccount, MAX_UINT64, ONE_SOL } from "./utils";
import { Creator } from "./interfaces";

describe("auction_house", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as Wallet;

  const auctionHouseProgram = anchor.workspace
    .AuctionHouse as Program<AuctionHouse>;
  const auctioneerProgram = anchor.workspace.Auctioneer as Program<Auctioneer>;
  const nftMinterProgram = anchor.workspace.NftMinter as Program<NftMinter>;

  const NAME = "Solana Course NFT";
  const SYMBOL = "SOLC";
  const URI =
    "https://raw.githubusercontent.com/arsenijkovalov/nft-assets/main/assets/nft.json";
  const SELLER_FEE_BASIS_POINTS = 10;
  const IS_MUTABLE = false;
  const MAX_SUPPLY = 0;

  const TOKEN_SIZE = 1;

  it("Test User Flow", async () => {
    const auctionHouse = await createAuctionHouse({
      auctionHouseProgram,
      auctioneerProgram,
      provider,
      payer,
    });

    const creators: Creator[] = [
      {
        address: payer.publicKey,
        share: 100,
        verified: false,
      },
    ];
    const token = await createNft({
      nftMinterProgram,
      payer,
      name: NAME,
      symbol: SYMBOL,
      uri: URI,
      creators,
      sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
      maxSupply: MAX_SUPPLY,
      isMutable: IS_MUTABLE,
    });

    const { sellAccounts } = await sell({
      auctioneerProgram,
      auctionHouse,
      token,
      startTime: Math.round(Date.now() / 1000),
      endTime: Math.round(Date.now() / 1000) + 10,
      tokenSize: TOKEN_SIZE,
    });

    const bidder0 = await createSystemAccount({ provider, payer });
    // The first bidder deposits 5 SOL to his escrow account
    const depositAmount = 5 * ONE_SOL;
    await deposit({
      auctioneerProgram,
      auctionHouse,
      token,
      buyerKeypair: bidder0,
      amount: depositAmount,
      tokenSize: TOKEN_SIZE,
    });
    // Then he withdraws 4 SOL from the escrow account
    const withdrawAmount = 4 * ONE_SOL;
    await withdraw({
      auctioneerProgram,
      auctionHouse,
      buyerKeypair: bidder0,
      amount: withdrawAmount,
    });
    // After all operations, makes his bid for 1 SOL
    const bid0Amount = depositAmount - withdrawAmount;
    await buy({
      auctioneerProgram,
      auctionHouse,
      token,
      buyerKeypair: bidder0,
      sellerAddress: sellAccounts.wallet,
      buyerPrice: bid0Amount,
      tokenSize: TOKEN_SIZE,
    });

    await sleep(500);

    const bidder1 = await createSystemAccount({ provider, payer });
    // The second bidder deposits 2 SOL to his escrow account
    const bid1Amount = 2 * ONE_SOL;
    await deposit({
      auctioneerProgram,
      auctionHouse,
      token,
      buyerKeypair: bidder1,
      amount: bid1Amount,
      tokenSize: TOKEN_SIZE,
    });
    // Then he bids all deposit amount (2 SOL)
    const { buyAccounts: bidder1BuyAccounts } = await buy({
      auctioneerProgram,
      auctionHouse,
      token,
      buyerKeypair: bidder1,
      sellerAddress: sellAccounts.wallet,
      buyerPrice: bid1Amount,
      tokenSize: TOKEN_SIZE,
    });

    console.log("Waiting for the end of the auction...");
    await sleep(10000);
    console.log("Auction is ended!");

    // After the end of the auction, the 2nd bid wins
    await executeSale({
      auctioneerProgram,
      auctionHouse,
      token,
      buyer: bidder1.publicKey,
      buyerPrice: bid1Amount,
      sellAccounts,
      buyAccounts: bidder1BuyAccounts,
      creators,
      tokenSize: TOKEN_SIZE,
      signerKeypair: auctionHouse.authorityKeypair,
      payer: payer,
    });
  });

  it("Cancel Listing", async () => {
    const auctionHouse = await createAuctionHouse({
      auctionHouseProgram,
      auctioneerProgram,
      provider,
      payer,
    });

    const token = await createNft({
      nftMinterProgram,
      payer,
      name: NAME,
      symbol: SYMBOL,
      uri: URI,
      sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
      maxSupply: MAX_SUPPLY,
      isMutable: IS_MUTABLE,
    });

    const { sellAccounts } = await sell({
      auctioneerProgram,
      auctionHouse,
      token,
      startTime: Math.round(Date.now() / 1000),
      endTime: Math.round(Date.now() / 1000) + 10,
      tokenSize: TOKEN_SIZE,
    });

    await cancel({
      auctioneerProgram,
      auctionHouse,
      token,
      walletKeypair: token.owner,
      tradeStateAddress: sellAccounts.sellerTradeState,
      buyerPrice: new BN(MAX_UINT64),
      tokenSize: TOKEN_SIZE,
    });
  });

  it("Cancel Bid", async () => {
    const auctionHouse = await createAuctionHouse({
      auctionHouseProgram,
      auctioneerProgram,
      provider,
      payer,
    });

    const token = await createNft({
      nftMinterProgram,
      payer,
      name: NAME,
      symbol: SYMBOL,
      uri: URI,
      sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
      maxSupply: MAX_SUPPLY,
      isMutable: IS_MUTABLE,
    });

    const { sellAccounts } = await sell({
      auctioneerProgram,
      auctionHouse,
      token,
      startTime: Math.round(Date.now() / 1000),
      endTime: Math.round(Date.now() / 1000) + 10,
      tokenSize: TOKEN_SIZE,
    });

    // First bidder

    const buyerKeypair = await createSystemAccount({ provider, payer });
    const depositAmount = 5 * ONE_SOL;
    await deposit({
      auctioneerProgram,
      auctionHouse,
      token,
      buyerKeypair,
      amount: depositAmount,
      tokenSize: TOKEN_SIZE,
    });

    await sleep(1000);

    const bidAmount = ONE_SOL;
    const { buyAccounts } = await buy({
      auctioneerProgram,
      auctionHouse,
      token,
      buyerKeypair,
      sellerAddress: sellAccounts.wallet,
      buyerPrice: bidAmount,
      tokenSize: TOKEN_SIZE,
    });

    // Second bidder

    const buyerKeypair2 = await createSystemAccount({ provider, payer });
    const depositAmount2 = 5 * ONE_SOL;
    await deposit({
      auctioneerProgram,
      auctionHouse,
      token,
      buyerKeypair: buyerKeypair2,
      amount: depositAmount2,
      tokenSize: TOKEN_SIZE,
    });

    await sleep(1000);

    const bidAmount2 = 2 * ONE_SOL;
    const {} = await buy({
      auctioneerProgram,
      auctionHouse,
      token,
      buyerKeypair: buyerKeypair2,
      sellerAddress: sellAccounts.wallet,
      buyerPrice: bidAmount2,
      tokenSize: TOKEN_SIZE,
    });

    await sleep(1000);

    await cancel({
      auctioneerProgram,
      auctionHouse,
      token,
      walletKeypair: buyerKeypair,
      tradeStateAddress: buyAccounts.buyerTradeState,
      buyerPrice: new BN(bidAmount),
      tokenSize: TOKEN_SIZE,
    });
  });

  it("Close Escrow Account", async () => {
    const auctionHouse = await createAuctionHouse({
      auctionHouseProgram,
      auctioneerProgram,
      provider,
      payer,
    });

    const token = await createNft({
      nftMinterProgram,
      payer,
      name: NAME,
      symbol: SYMBOL,
      uri: URI,
      sellerFeeBasisPoints: SELLER_FEE_BASIS_POINTS,
      maxSupply: MAX_SUPPLY,
      isMutable: IS_MUTABLE,
    });

    const buyerKeypair = await createSystemAccount({ provider, payer });

    const depositAmount = 2 * ONE_SOL;
    await deposit({
      auctioneerProgram,
      auctionHouse,
      token,
      buyerKeypair,
      amount: depositAmount,
      tokenSize: TOKEN_SIZE,
    });

    await closeEscrowAccount({
      auctionHouseProgram,
      auctionHouse,
      walletKeypair: buyerKeypair,
    });
  });

  it("Withdraw From Fee", async () => {
    const auctionHouse = await createAuctionHouse({
      auctionHouseProgram,
      auctioneerProgram,
      provider,
      payer,
    });

    const auctionHouseFeeAccountBalance = ONE_SOL;
    const transferTx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: auctionHouse.auctionHouseFeeAccount,
        lamports: auctionHouseFeeAccountBalance,
      })
    );
    await provider.sendAndConfirm(transferTx, [payer.payer]);

    await withdrawFromFee({
      auctionHouseProgram,
      auctionHouse,
      amount: auctionHouseFeeAccountBalance,
    });
  });

  it("Withdraw From Treasury", async () => {
    const auctionHouse = await createAuctionHouse({
      auctionHouseProgram,
      auctioneerProgram,
      provider,
      payer,
    });

    const auctionHouseTreasuryBalance = ONE_SOL;
    const transferTx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: auctionHouse.auctionHouseTreasury,
        lamports: auctionHouseTreasuryBalance,
      })
    );
    await provider.sendAndConfirm(transferTx, [payer.payer]);

    await withdrawFromTreasury({
      auctionHouseProgram,
      auctionHouse,
      amount: auctionHouseTreasuryBalance,
    });
  });
});
