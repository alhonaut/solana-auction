use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use auctioneer::errors::AuctioneerError;
use solana_program_test::tokio;
use std::time::SystemTime;

mod utils;
use utils::setup_functions::*;

#[tokio::test]
async fn cancel_sell_success() {
    let mut context = auctioneer_program_test().start_with_context().await;

    let (_, auction_house, auction_house_data) = create_auction_house(&mut context, 100, false)
        .await
        .expect("Failed to create Auction House");

    let token = create_nft(&mut context, None)
        .await
        .expect("Failed to create NFT");

    // Sell
    let (sell_accounts, sell_tx) = sell(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
        None,
        None,
        None,
        None,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .expect("Failed to sell NFT");

    // CancelSell
    let (_, cancel_sell_tx) = cancel_sell(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        &sell_accounts.seller_trade_state,
    );
    context
        .banks_client
        .process_transaction(cancel_sell_tx)
        .await
        .unwrap();

    let listing_config_account = context
        .banks_client
        .get_account(sell_accounts.listing_config)
        .await
        .unwrap();

    // Assert listing config account is closed
    assert!(listing_config_account.is_none());
}

#[tokio::test]
async fn failure_cancel_buy_highest_bid() {
    let mut context = auctioneer_program_test().start_with_context().await;

    let (_, auction_house, auction_house_data) = create_auction_house(&mut context, 100, false)
        .await
        .expect("Failed to create Auction House");

    let token = create_nft(&mut context, None)
        .await
        .expect("Failed to create NFT");

    // Sell

    let (_, sell_tx) = sell(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
        None,
        None,
        None,
        None,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .expect("Failed to sell NFT");

    // Buy 1

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let (_, deposit_tx) = deposit(
        &mut context,
        &auction_house,
        &auction_house_data,
        &buyer,
        5 * ONE_SOL,
    );
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let bid_amount = ONE_SOL;

    let (_, buy_tx) = buy(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        &buyer,
        bid_amount,
    );
    context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap();

    // Buy 2

    let buyer2 = Keypair::new();
    airdrop(&mut context, &buyer2.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let (_, deposit_tx2) = deposit(
        &mut context,
        &auction_house,
        &auction_house_data,
        &buyer2,
        5 * ONE_SOL,
    );
    context
        .banks_client
        .process_transaction(deposit_tx2)
        .await
        .unwrap();

    let bid_amount2 = 2 * ONE_SOL;

    let (buy_accounts2, buy_tx2) = buy(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        &buyer2,
        bid_amount2,
    );
    context
        .banks_client
        .process_transaction(buy_tx2)
        .await
        .unwrap();

    // CancelBuy

    let (_, cancel_buy_tx) = cancel_buy(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        &buyer2,
        &buy_accounts2.buyer_trade_state,
        bid_amount2,
    );

    let tx_error = context
        .banks_client
        .process_transaction(cancel_buy_tx)
        .await
        .unwrap_err();

    assert_error(tx_error, AuctioneerError::CannotCancelHighestBid.into());
}
