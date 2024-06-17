use anchor_client::solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use anchor_lang::{
    solana_program::{instruction::Instruction, system_program},
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use auction_house::{errors::AuctionHouseError, pda::find_auctioneer_address, AuctionHouse};
use solana_program_test::tokio;

mod utils;
use utils::setup_functions::*;

#[tokio::test]
async fn delegate_auctioneer_success() {
    let mut context = auction_house_program_test().start_with_context().await;

    let (authority, auction_house, _) = create_auction_house(&mut context, 100, false)
        .await
        .expect("Failed to create Auction House");

    // DelegateAuctioneer

    let auctioneer_authority = Keypair::new();
    let (auctioneer, _) = find_auctioneer_address(&auction_house, &auctioneer_authority.pubkey());

    let data = auction_house::instruction::DelegateAuctioneer {};

    let accounts = auction_house::accounts::DelegateAuctioneer {
        auction_house,
        authority: authority.pubkey(),
        auctioneer_authority: auctioneer_authority.pubkey(),
        auctioneer,
        system_program: system_program::id(),
    };

    let ix = Instruction {
        program_id: auction_house::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let auction_house_account = context
        .banks_client
        .get_account(auction_house)
        .await
        .unwrap()
        .expect("Auction House account not found");

    let auction_house_data =
        AuctionHouse::try_deserialize(&mut auction_house_account.data.as_ref()).unwrap();

    assert_eq!(auction_house_data.authority, authority.pubkey());
    assert!(auction_house_data.has_auctioneer);
}

#[tokio::test]
async fn failure_delegate_auctioneer_redelegate() {
    let mut context = auction_house_program_test().start_with_context().await;

    let (authority, auction_house, _) = create_auction_house(&mut context, 100, false)
        .await
        .expect("Failed to create Auction House");

    // DelegateAuctioneer

    let auctioneer_authority = Keypair::new();
    let (auctioneer, _) = find_auctioneer_address(&auction_house, &auctioneer_authority.pubkey());

    let tx = delegate(
        &mut context,
        &auction_house,
        &authority,
        &auctioneer_authority.pubkey(),
        &auctioneer,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Auctioneer redelegation

    let new_auctioneer_authority = Keypair::new();
    let (new_auctioneer, _) =
        find_auctioneer_address(&auction_house, &new_auctioneer_authority.pubkey());

    let tx = delegate(
        &mut context,
        &auction_house,
        &authority,
        &new_auctioneer_authority.pubkey(),
        &new_auctioneer,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(
        tx_error,
        AuctionHouseError::AuctionHouseAlreadyDelegated.into(),
    );
}
