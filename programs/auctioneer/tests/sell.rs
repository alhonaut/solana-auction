use anchor_client::solana_sdk::{signer::Signer, transaction::Transaction};
use anchor_lang::{
    prelude::Pubkey,
    solana_program::{instruction::Instruction, system_program, sysvar},
    InstructionData, ToAccountMetas,
};
use anchor_spl::token::spl_token;
use auction_house::pda::*;
use auctioneer::pda::*;
use solana_program_test::tokio;
use std::time::SystemTime;

mod utils;
use utils::setup_functions::*;

#[tokio::test]
async fn sell_success() {
    let mut context = auctioneer_program_test().start_with_context().await;

    let (_, auction_house, auction_house_data) = create_auction_house(&mut context, 100, false)
        .await
        .expect("Failed to create Auction House");

    let token = create_nft(&mut context, None)
        .await
        .expect("Failed to create NFT");

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

    let seller_trade_state_account = context
        .banks_client
        .get_account(sell_accounts.seller_trade_state)
        .await
        .expect("Account not found")
        .expect("Account is empty");

    assert_eq!(seller_trade_state_account.data.len(), 1);
}

#[tokio::test]
async fn failure_sell_wrong_auctioneer_authority() {
    let mut context = auctioneer_program_test().start_with_context().await;

    let (_, auction_house, auction_house_data) = create_auction_house(&mut context, 100, false)
        .await
        .expect("Failed to create Auction House");

    let token = create_nft(&mut context, None)
        .await
        .expect("Failed to create NFT");

    let (seller_trade_state, seller_trade_state_bump) = find_auctioneer_trade_state_address(
        &token.owner.pubkey(),
        &auction_house,
        &token.ata,
        &auction_house_data.treasury_mint,
        &token.mint.pubkey(),
        1,
    );

    let (free_seller_trade_state, free_seller_trade_state_bump) = find_trade_state_address(
        &token.owner.pubkey(),
        &auction_house,
        &token.ata,
        &auction_house_data.treasury_mint,
        &token.mint.pubkey(),
        0,
        1,
    );

    let (listing_config, _) = find_listing_config_address(
        &token.owner.pubkey(),
        &auction_house,
        &token.ata,
        &auction_house_data.treasury_mint,
        &token.mint.pubkey(),
        1,
    );

    let (program_as_signer, program_as_signer_bump) = find_program_as_signer_address();

    // Delegate external auctioneer authority
    let (wrong_auctioneer_authority, wrong_auctioneer_authority_bump) =
        Pubkey::find_program_address(
            &["not_auctioneer".as_bytes(), auction_house.as_ref()],
            &auctioneer::id(),
        );

    let (auctioneer, _) = find_auctioneer_address(&auction_house, &wrong_auctioneer_authority);

    let data = auctioneer::instruction::Sell {
        trade_state_bump: seller_trade_state_bump,
        free_trade_state_bump: free_seller_trade_state_bump,
        program_as_signer_bump,
        auctioneer_authority_bump: wrong_auctioneer_authority_bump,
        token_size: 1,
        start_time: (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 60) as i64,
        end_time: (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60) as i64,
        reserve_price: None,
        min_bid_increment: None,
        time_ext_period: None,
        time_ext_delta: None,
    };

    let accounts = auctioneer::accounts::AuctioneerSell {
        auction_house_program: auction_house::id(),
        listing_config,
        wallet: token.owner.pubkey(),
        token_account: token.ata,
        metadata: token.metadata,
        authority: auction_house_data.authority,
        auction_house,
        auction_house_fee_account: auction_house_data.auction_house_fee_account,
        seller_trade_state,
        free_seller_trade_state,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        program_as_signer,
        rent: sysvar::rent::id(),
        auctioneer_authority: wrong_auctioneer_authority,
        auctioneer,
    };

    let ix = Instruction {
        program_id: auctioneer::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&token.owner.pubkey()),
        &[&token.owner],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ERR_ACCOUNT_NOT_INITIALIZED);
}
