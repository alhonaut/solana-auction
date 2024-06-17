use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_spl::token::spl_token;
use auctioneer::errors::AuctioneerError;
use nft_minter::utils::Creator;
use solana_program_test::tokio;
use std::time::SystemTime;

mod utils;
use utils::setup_functions::*;

#[tokio::test]
async fn execute_sale_success() {
    let mut context = auctioneer_program_test().start_with_context().await;

    let (authority, auction_house, auction_house_data) =
        create_auction_house(&mut context, 100, false)
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

    let (buy_accounts, buy_tx) = buy(
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

    context.warp_to_slot(120 * 400).unwrap();

    // Execute sale

    let (_, execute_sale_tx) = execute_sale(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        None,
        &sell_accounts,
        &buy_accounts,
        bid_amount,
        &authority, // Any of the following can be a signer-payer: auction winner (buyer with the highest bid),
                    // token seller (token owner) or Auction House authority.
                    // NOTE: If Auction House authority is the signer-payer, Auction House Fee Account must have enough balance
                    // to proceed the transaction
    )
    .await;
    context
        .banks_client
        .process_transaction(execute_sale_tx)
        .await
        .unwrap();
}

#[tokio::test]
async fn execute_sale_multiple_buy_success() {
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

    let (_buy_accounts, buy_tx) = buy(
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

    context.warp_to_slot(120 * 400).unwrap();

    // Execute sale

    let (_, execute_sale_tx) = execute_sale(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        None,
        &sell_accounts,
        &buy_accounts2,
        bid_amount2,
        &token.owner, // Any of the following can be a signer-payer: auction winner (buyer with the highest bid),
                      // token seller (token owner) or Auction House authority.
                      // NOTE: If Auction House authority is the signer-payer, Auction House Fee Account must have enough balance
                      // to proceed the transaction
    )
    .await;
    context
        .banks_client
        .process_transaction(execute_sale_tx)
        .await
        .unwrap();
}

#[tokio::test]
async fn failure_execute_sale_multiple_buy_not_highest_bid() {
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

    let (buy_accounts, buy_tx) = buy(
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

    let (_buy_accounts2, buy_tx2) = buy(
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

    context.warp_to_slot(120 * 400).unwrap();

    // Execute sale

    let (_, execute_sale_tx) = execute_sale(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        None,
        &sell_accounts,
        &buy_accounts, // Not highest bidder accounts
        bid_amount,    // Not highest bid
        &buyer, // Any of the following can be a signer-payer: auction winner (buyer with the highest bid),
                // token seller (token owner) or Auction House authority.
                // NOTE: If Auction House authority is the signer-payer, Auction House Fee Account must have enough balance
                // to proceed the transaction
    )
    .await;

    let tx_error = context
        .banks_client
        .process_transaction(execute_sale_tx)
        .await
        .unwrap_err();

    assert_error(tx_error, AuctioneerError::NotHighestBidder.into());
}

#[tokio::test]
async fn execute_sale_with_metadata_creators_success() {
    let mut context = auctioneer_program_test().start_with_context().await;

    let (authority, auction_house, auction_house_data) =
        create_auction_house(&mut context, 100, false)
            .await
            .expect("Failed to create Auction House");

    let metadata_creators = vec![
        Creator {
            address: Pubkey::new_unique(),
            verified: false,
            share: 25,
        },
        Creator {
            address: Pubkey::new_unique(),
            verified: false,
            share: 75,
        },
    ];

    for creator in &metadata_creators {
        // airdrop 0.1 sol to ensure rent-exempt minimum
        airdrop(&mut context, &creator.address, 100_000_000)
            .await
            .unwrap();
    }

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

    let (buy_accounts, buy_tx) = buy(
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

    context.warp_to_slot(120 * 400).unwrap();

    // Execute sale

    let (execute_sale_accounts, execute_sale_tx) = execute_sale(
        &mut context,
        &auction_house,
        &auction_house_data,
        &token,
        None,
        &sell_accounts,
        &buy_accounts,
        bid_amount,
        &authority, // Any of the following can be a signer-payer: auction winner (buyer with the highest bid),
                    // token seller (token owner) or Auction House authority.
                    // NOTE: If Auction House authority is the signer-payer, Auction House Fee Account must have enough balance
                    // to proceed the transaction
    )
    .await;

    let seller_before = context
        .banks_client
        .get_account(token.owner.pubkey())
        .await
        .unwrap()
        .unwrap();

    let mut metadata_creators_before: Vec<anchor_client::solana_sdk::account::Account> = Vec::new();
    for creator in &metadata_creators {
        metadata_creators_before.push(
            context
                .banks_client
                .get_account(creator.address)
                .await
                .unwrap()
                .unwrap(),
        );
    }

    let buyer_token_before = &context
        .banks_client
        .get_account(execute_sale_accounts.buyer_receipt_token_account)
        .await
        .unwrap();

    // Assert that account is empty
    assert!(buyer_token_before.is_none());

    // Executing the sale
    context
        .banks_client
        .process_transaction(execute_sale_tx)
        .await
        .unwrap();

    let seller_after = context
        .banks_client
        .get_account(token.owner.pubkey())
        .await
        .unwrap()
        .unwrap();

    let mut metadata_creators_after: Vec<anchor_client::solana_sdk::account::Account> = Vec::new();
    for creator in &metadata_creators {
        metadata_creators_after.push(
            context
                .banks_client
                .get_account(creator.address)
                .await
                .unwrap()
                .unwrap(),
        );
    }
    let buyer_token_after = crate::spl_token::state::Account::unpack_from_slice(
        context
            .banks_client
            .get_account(execute_sale_accounts.buyer_receipt_token_account)
            .await
            .unwrap()
            .unwrap()
            .data
            .as_slice(),
    )
    .unwrap();

    let listing_config_closed = context
        .banks_client
        .get_account(sell_accounts.listing_config)
        .await
        .unwrap();

    // Assert that listing account is closed
    assert!(listing_config_closed.is_none());

    assert!(seller_before.lamports < seller_after.lamports);
    assert_eq!(buyer_token_after.amount, 1);
}
