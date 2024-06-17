use anchor_client::solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use anchor_lang::{
    solana_program::{instruction::Instruction, system_program, sysvar},
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use anchor_spl::associated_token;
use auction_house::{pda::*, AuctionHouse};
use solana_program_test::tokio;

mod utils;
use utils::setup_functions::*;

#[tokio::test]
async fn create_auction_house_success() {
    let mut context = auction_house_program_test().start_with_context().await;

    let authority = Keypair::new();
    airdrop(&mut context, &authority.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let treasury_mint = spl_token::native_mint::id();

    let (auction_house, auction_house_bump) =
        find_auction_house_address(&authority.pubkey(), &treasury_mint);
    let (auction_house_fee_account, auction_house_fee_account_bump) =
        find_auction_house_fee_account_address(&auction_house);
    let (auction_house_treasury, auction_house_treasury_bump) =
        find_auction_house_treasury_address(&auction_house);

    let data = auction_house::instruction::CreateAuctionHouse {
        _bump: auction_house_bump,
        fee_payer_bump: auction_house_fee_account_bump,
        treasury_bump: auction_house_treasury_bump,
        seller_fee_basis_points: 10,
        can_change_sale_price: false,
    };

    let accounts = auction_house::accounts::CreateAuctionHouse {
        treasury_mint,
        payer: authority.pubkey(),
        authority: authority.pubkey(),
        fee_withdrawal_destination: context.payer.pubkey(),
        treasury_withdrawal_destination: context.payer.pubkey(),
        treasury_withdrawal_destination_owner: context.payer.pubkey(),
        auction_house,
        auction_house_fee_account,
        auction_house_treasury,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        associated_token_program: associated_token::ID,
        rent: sysvar::rent::id(),
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
}

#[tokio::test]
async fn failure_create_auction_house_reinitialization() {
    let mut context = auction_house_program_test().start_with_context().await;

    // CreateAuctionHouse

    let authority = Keypair::new();
    airdrop(&mut context, &authority.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let treasury_mint = spl_token::native_mint::id();

    let (auction_house, auction_house_bump) =
        find_auction_house_address(&authority.pubkey(), &treasury_mint);
    let (auction_house_fee_account, auction_house_fee_account_bump) =
        find_auction_house_fee_account_address(&auction_house);
    let (auction_house_treasury, auction_house_treasury_bump) =
        find_auction_house_treasury_address(&auction_house);

    let data = auction_house::instruction::CreateAuctionHouse {
        _bump: auction_house_bump,
        fee_payer_bump: auction_house_fee_account_bump,
        treasury_bump: auction_house_treasury_bump,
        seller_fee_basis_points: 10,
        can_change_sale_price: false,
    };

    let accounts = auction_house::accounts::CreateAuctionHouse {
        treasury_mint,
        payer: authority.pubkey(),
        authority: authority.pubkey(),
        fee_withdrawal_destination: context.payer.pubkey(),
        treasury_withdrawal_destination: context.payer.pubkey(),
        treasury_withdrawal_destination_owner: context.payer.pubkey(),
        auction_house,
        auction_house_fee_account,
        auction_house_treasury,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        associated_token_program: associated_token::ID,
        rent: sysvar::rent::id(),
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

    // Attempt to reinitialize (Hack)

    let malicious_wallet = Keypair::new();
    airdrop(&mut context, &malicious_wallet.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let hacked_fee_withdrawal_destination = malicious_wallet.pubkey();
    let hacked_treasury_withdrawal_destination = malicious_wallet.pubkey();
    let hacked_treasury_withdrawal_destination_owner = malicious_wallet.pubkey();

    let data = auction_house::instruction::CreateAuctionHouse {
        _bump: auction_house_bump,
        fee_payer_bump: auction_house_fee_account_bump,
        treasury_bump: auction_house_treasury_bump,
        seller_fee_basis_points: 10,
        can_change_sale_price: false,
    };

    let accounts = auction_house::accounts::CreateAuctionHouse {
        treasury_mint,
        payer: authority.pubkey(),
        authority: authority.pubkey(),
        fee_withdrawal_destination: hacked_fee_withdrawal_destination,
        treasury_withdrawal_destination: hacked_treasury_withdrawal_destination,
        treasury_withdrawal_destination_owner: hacked_treasury_withdrawal_destination_owner,
        auction_house,
        auction_house_fee_account,
        auction_house_treasury,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        associated_token_program: associated_token::ID,
        rent: sysvar::rent::id(),
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

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ERR_AUCTION_HOUSE_ALREADY_INITIALIZED);
}
