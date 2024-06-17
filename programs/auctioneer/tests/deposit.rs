use anchor_client::solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use anchor_lang::{
    prelude::Pubkey,
    solana_program::{instruction::Instruction, system_program, sysvar},
    InstructionData, ToAccountMetas,
};
use anchor_spl::token::spl_token;
use auction_house::pda::*;
use solana_program_test::tokio;

mod utils;
use utils::setup_functions::*;

#[tokio::test]
async fn deposit_success() {
    let mut context = auctioneer_program_test().start_with_context().await;

    let (_, auction_house, auction_house_data) = create_auction_house(&mut context, 100, false)
        .await
        .expect("Failed to create Auction House");

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 2 * ONE_SOL)
        .await
        .unwrap();

    let (deposit_accounts, deposit_tx) = deposit(
        &mut context,
        &auction_house,
        &auction_house_data,
        &buyer,
        ONE_SOL,
    );
    context
        .banks_client
        .process_transaction(deposit_tx)
        .await
        .unwrap();

    let escrow_payment_account_data_len = 0;
    let rent = context.banks_client.get_rent().await.unwrap();
    let rent_exempt_min: u64 = rent.minimum_balance(escrow_payment_account_data_len);

    let escrow_payment_account = context
        .banks_client
        .get_account(deposit_accounts.escrow_payment_account)
        .await
        .expect("Account not found")
        .expect("Account is empty");

    assert_eq!(escrow_payment_account.lamports, ONE_SOL + rent_exempt_min);
}

#[tokio::test]
async fn failure_deposit_wrong_auctioneer_authority() {
    let mut context = auctioneer_program_test().start_with_context().await;

    let (_, auction_house, auction_house_data) = create_auction_house(&mut context, 100, false)
        .await
        .expect("Failed to create Auction House");

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 2 * ONE_SOL)
        .await
        .unwrap();

    let (escrow_payment_account, escrow_payment_account_bump) =
        find_escrow_payment_account_address(&auction_house, &buyer.pubkey());

    // Delegate external auctioneer authority
    let (wrong_auctioneer_authority, wrong_auctioneer_authority_bump) =
        Pubkey::find_program_address(
            &["not_auctioneer".as_bytes(), auction_house.as_ref()],
            &auctioneer::id(),
        );

    let (auctioneer, _) = find_auctioneer_address(&auction_house, &wrong_auctioneer_authority);

    let data = auctioneer::instruction::Deposit {
        amount: ONE_SOL,
        escrow_payment_bump: escrow_payment_account_bump,
        auctioneer_authority_bump: wrong_auctioneer_authority_bump,
    };

    let accounts = auctioneer::accounts::AuctioneerDeposit {
        auction_house_program: auction_house::id(),
        wallet: buyer.pubkey(),
        authority: auction_house_data.authority,
        auction_house,
        auction_house_fee_account: auction_house_data.auction_house_fee_account,
        token_program: spl_token::id(),
        treasury_mint: auction_house_data.treasury_mint,
        payment_account: buyer.pubkey(),
        transfer_authority: buyer.pubkey(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
        escrow_payment_account,
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
        Some(&buyer.pubkey()),
        &[&buyer],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ERR_ACCOUNT_NOT_INITIALIZED);
}
