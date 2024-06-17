use anchor_client::solana_sdk::{signer::Signer, transaction::Transaction};
use anchor_lang::{
    solana_program::{instruction::Instruction, system_program},
    InstructionData, ToAccountMetas,
};
use solana_program_test::tokio;

mod utils;
use utils::setup_functions::*;

#[tokio::test]
async fn withdraw_from_fee_success() {
    let mut context = auction_house_program_test().start_with_context().await;

    let (authority, auction_house, auction_house_data) =
        create_auction_house(&mut context, 100, false)
            .await
            .expect("Failed to create Auction House");

    // Airdrop to Auction House fee account

    let amount = ONE_SOL;
    airdrop(
        &mut context,
        &auction_house_data.auction_house_fee_account,
        amount,
    )
    .await
    .unwrap();

    let fee_account_before = context
        .banks_client
        .get_account(auction_house_data.auction_house_fee_account)
        .await
        .unwrap()
        .unwrap();

    // WithdrawFromFee

    let data = auction_house::instruction::WithdrawFromFee { amount };

    let accounts = auction_house::accounts::WithdrawFromFee {
        authority: auction_house_data.authority,
        fee_withdrawal_destination: auction_house_data.fee_withdrawal_destination,
        auction_house_fee_account: auction_house_data.auction_house_fee_account,
        auction_house,
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

    let fee_account_after = context
        .banks_client
        .get_account(auction_house_data.auction_house_treasury)
        .await
        .unwrap();

    assert_eq!(fee_account_before.lamports, amount);
    assert!(fee_account_after.is_none());
}

#[tokio::test]
async fn failure_withdraw_from_fee_insufficient_funds() {
    let mut context = auction_house_program_test().start_with_context().await;

    let (authority, auction_house, auction_house_data) =
        create_auction_house(&mut context, 100, false)
            .await
            .expect("Failed to create Auction House");

    // Airdrop to Auction House fee account

    let airdrop_amount = ONE_SOL;
    airdrop(
        &mut context,
        &auction_house_data.auction_house_fee_account,
        airdrop_amount,
    )
    .await
    .unwrap();

    // WithdrawFromFee

    let withdraw_amount = airdrop_amount + 1;

    let data = auction_house::instruction::WithdrawFromFee {
        amount: withdraw_amount,
    };

    let accounts = auction_house::accounts::WithdrawFromFee {
        authority: auction_house_data.authority,
        fee_withdrawal_destination: auction_house_data.fee_withdrawal_destination,
        auction_house_fee_account: auction_house_data.auction_house_fee_account,
        auction_house,
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

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ERR_INSUFFICIENT_FUNDS);
}
