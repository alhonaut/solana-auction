use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use auction_house::errors::AuctionHouseError;
use solana_program_test::tokio;

mod utils;
use utils::setup_functions::*;

#[tokio::test]
async fn withdraw_success() {
    let mut context = auctioneer_program_test().start_with_context().await;

    let (_, auction_house, auction_house_data) = create_auction_house(&mut context, 100, false)
        .await
        .expect("Failed to create Auction House");

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 3 * ONE_SOL)
        .await
        .unwrap();

    let deposit_amount = 2 * ONE_SOL;
    let (deposit_accounts, deposit_tx) = deposit(
        &mut context,
        &auction_house,
        &auction_house_data,
        &buyer,
        deposit_amount,
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

    assert_eq!(
        escrow_payment_account.lamports,
        deposit_amount + rent_exempt_min
    );

    let withdraw_amount = deposit_amount - ONE_SOL;
    let (_, withdraw_tx) = withdraw(
        &mut context,
        &auction_house,
        &auction_house_data,
        &buyer,
        withdraw_amount,
    );
    context
        .banks_client
        .process_transaction(withdraw_tx)
        .await
        .unwrap();

    let escrow_payment_account_after_withdraw = context
        .banks_client
        .get_account(deposit_accounts.escrow_payment_account)
        .await
        .expect("Account not found")
        .expect("Account is empty");

    assert_eq!(
        escrow_payment_account_after_withdraw.lamports,
        deposit_amount + rent_exempt_min - withdraw_amount
    );
}

#[tokio::test]
async fn failure_withdraw() {
    let mut context = auctioneer_program_test().start_with_context().await;

    let (_, auction_house, auction_house_data) = create_auction_house(&mut context, 100, false)
        .await
        .expect("Failed to create Auction House");

    let buyer = Keypair::new();
    airdrop(&mut context, &buyer.pubkey(), 3 * ONE_SOL)
        .await
        .unwrap();

    let deposit_amount = 2 * ONE_SOL;
    let (deposit_accounts, deposit_tx) = deposit(
        &mut context,
        &auction_house,
        &auction_house_data,
        &buyer,
        deposit_amount,
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

    assert_eq!(
        escrow_payment_account.lamports,
        deposit_amount + rent_exempt_min
    );

    let withdraw_amount = deposit_amount + rent_exempt_min + 1;
    let (_, withdraw_tx) = withdraw(
        &mut context,
        &auction_house,
        &auction_house_data,
        &buyer,
        withdraw_amount,
    );

    let tx_error = context
        .banks_client
        .process_transaction(withdraw_tx)
        .await
        .unwrap_err();

    assert_error(tx_error, AuctionHouseError::InsufficientFunds.into());
}
