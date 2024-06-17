use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_lang::AccountDeserialize;
use auctioneer::errors::AuctioneerError;
use auctioneer::sell::config::ListingConfig;
use solana_program_test::tokio;
use std::time::SystemTime;

mod utils;
use utils::setup_functions::*;

#[tokio::test]
async fn buy_success() {
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

    // Buy

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

    let listing_config_account = context
        .banks_client
        .get_account(sell_accounts.listing_config)
        .await
        .unwrap()
        .unwrap()
        .data;

    let listing_config_data =
        ListingConfig::try_deserialize(&mut listing_config_account.as_ref()).unwrap();

    assert_eq!(listing_config_data.highest_bid.amount, bid_amount);
}

#[tokio::test]
async fn multiple_buy_success() {
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

    // Buy 1

    let buyer1 = Keypair::new();
    airdrop(&mut context, &buyer1.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let (_, deposit_tx1) = deposit(
        &mut context,
        &auction_house,
        &auction_house_data,
        &buyer1,
        5 * ONE_SOL,
    );
    context
        .banks_client
        .process_transaction(deposit_tx1)
        .await
        .unwrap();

    let bid_amount1 = ONE_SOL;

    let (_, buy_tx1) = buy(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        &buyer1,
        bid_amount1,
    );
    context
        .banks_client
        .process_transaction(buy_tx1)
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

    let (_, buy_tx2) = buy(
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

    let listing_config_account = context
        .banks_client
        .get_account(sell_accounts.listing_config)
        .await
        .unwrap()
        .unwrap()
        .data;

    let listing_config_data =
        ListingConfig::try_deserialize(&mut listing_config_account.as_ref()).unwrap();

    assert_eq!(listing_config_data.highest_bid.amount, bid_amount2);
}

#[tokio::test]
async fn failure_buy_below_reserve_price() {
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
        Some(ONE_SOL + 1), // Reserve price
        None,
        None,
        None,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .expect("Failed to sell NFT");

    // Buy

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

    let bid_amount = ONE_SOL; // Not passes reserve price

    let (_, buy_tx) = buy(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        &buyer,
        bid_amount,
    );

    let tx_error = context
        .banks_client
        .process_transaction(buy_tx)
        .await
        .unwrap_err();

    assert_error(tx_error, AuctioneerError::BelowReservePrice.into());
}

#[tokio::test]
async fn failure_multiple_buy_increment() {
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
        Some(3 * ONE_SOL), // Min bid increment
        None,
        None,
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .expect("Failed to sell NFT");

    // Buy 1

    let buyer1 = Keypair::new();
    airdrop(&mut context, &buyer1.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let (_, deposit_tx1) = deposit(
        &mut context,
        &auction_house,
        &auction_house_data,
        &buyer1,
        5 * ONE_SOL,
    );
    context
        .banks_client
        .process_transaction(deposit_tx1)
        .await
        .unwrap();

    let bid_amount1 = 3 * ONE_SOL; // Passes minimum bid increment

    let (_, buy_tx1) = buy(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        &buyer1,
        bid_amount1,
    );
    context
        .banks_client
        .process_transaction(buy_tx1)
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

    let bid_amount2 = 4 * ONE_SOL; // Not passes minimum bid increment

    let (_, buy_tx2) = buy(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        &buyer2,
        bid_amount2,
    );

    let tx_error = context
        .banks_client
        .process_transaction(buy_tx2)
        .await
        .unwrap_err();

    assert_error(tx_error, AuctioneerError::BelowBidIncrement.into());
}

#[tokio::test]
async fn multiple_buy_time_ext_success() {
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
        Some(60),
        Some(60),
    );
    context
        .banks_client
        .process_transaction(sell_tx)
        .await
        .expect("Failed to sell NFT");

    context.warp_to_slot(400).unwrap();

    let listing_config_account0 = context
        .banks_client
        .get_account(sell_accounts.listing_config)
        .await
        .unwrap()
        .unwrap()
        .data;

    let listing_config_data0 =
        ListingConfig::try_deserialize(&mut listing_config_account0.as_ref()).unwrap();

    // Actual time before auction ending
    let end_time0 = listing_config_data0.end_time;

    // Buy 1

    let buyer1 = Keypair::new();
    airdrop(&mut context, &buyer1.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let (_, deposit_tx1) = deposit(
        &mut context,
        &auction_house,
        &auction_house_data,
        &buyer1,
        5 * ONE_SOL,
    );
    context
        .banks_client
        .process_transaction(deposit_tx1)
        .await
        .unwrap();

    let bid_amount1 = ONE_SOL;

    let (_, buy_tx1) = buy(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        &buyer1,
        bid_amount1,
    );
    context
        .banks_client
        .process_transaction(buy_tx1)
        .await
        .unwrap();

    let listing_config_account1 = context
        .banks_client
        .get_account(sell_accounts.listing_config)
        .await
        .unwrap()
        .unwrap()
        .data;

    let listing_config_data1 =
        ListingConfig::try_deserialize(&mut listing_config_account1.as_ref()).unwrap();

    // Assert new expanded time before auction ending
    assert_eq!(listing_config_data1.end_time, end_time0 + 60);

    context.warp_to_slot(121 * 400).unwrap();

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

    let (_, buy_tx2) = buy(
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

    let listing_config_account2 = context
        .banks_client
        .get_account(sell_accounts.listing_config)
        .await
        .unwrap()
        .unwrap()
        .data;

    let listing_config_data2 =
        ListingConfig::try_deserialize(&mut listing_config_account2.as_ref()).unwrap();

    // Assert new expanded time before auction ending
    assert_eq!(listing_config_data2.end_time, end_time0 + 60 + 60);
}
