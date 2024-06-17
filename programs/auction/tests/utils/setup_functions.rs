#![allow(dead_code)]

use anchor_client::solana_sdk::transaction::{Transaction, TransactionError};
use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_lang::solana_program::instruction::{Instruction, InstructionError};
use anchor_lang::solana_program::{system_instruction, system_program, sysvar};
use anchor_lang::{prelude::Pubkey, AccountDeserialize};
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::{self};
use anchor_spl::token::spl_token;
use auction_house::pda::*;
use auction_house::AuctionHouse;
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use std::io;

// Error = Error code
pub const ERR_AUCTION_HOUSE_ALREADY_INITIALIZED: u32 = 0;
pub const ERR_INSUFFICIENT_FUNDS: u32 = 1;

pub const ONE_SOL: u64 = 1_000_000_000;

pub fn auction_house_program_test() -> ProgramTest {
    let program = ProgramTest::new("auction_house", auction_house::id(), None);
    program
}

pub fn assert_error(error: BanksClientError, expected_error: u32) {
    match error {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::Custom(e),
        )) => assert_eq!(e, expected_error),
        _ => assert!(false),
    }
}

pub async fn create_auction_house(
    context: &mut ProgramTestContext,
    seller_fee_basis_points: u16,
    can_change_sale_price: bool,
) -> Result<(Keypair, Pubkey, AuctionHouse), BanksClientError> {
    // CreateAuctionHouse
    let authority = Keypair::new();
    airdrop(context, &authority.pubkey(), 10 * ONE_SOL).await?;

    let treasury_mint = spl_token::native_mint::id();

    let (auction_house, auction_house_bump) =
        find_auction_house_address(&authority.pubkey(), &treasury_mint);
    let (auction_house_fee_account, auction_house_fee_account_bump) =
        find_auction_house_fee_account_address(&auction_house);
    let (auction_house_treasury, auction_house_treasury_bump) =
        find_auction_house_treasury_address(&auction_house);

    let create_auction_house_ix = Instruction {
        program_id: auction_house::id(),
        data: auction_house::instruction::CreateAuctionHouse {
            _bump: auction_house_bump,
            fee_payer_bump: auction_house_fee_account_bump,
            treasury_bump: auction_house_treasury_bump,
            seller_fee_basis_points,
            can_change_sale_price,
        }
        .data(),
        accounts: auction_house::accounts::CreateAuctionHouse {
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
        }
        .to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_auction_house_ix],
        Some(&authority.pubkey()),
        &[&authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await?;

    let auction_house_account = context
        .banks_client
        .get_account(auction_house)
        .await?
        .expect("Auction House account not found");
    let auction_house_data =
        AuctionHouse::try_deserialize(&mut auction_house_account.data.as_ref())
            .map_err(|e| BanksClientError::Io(io::Error::new(io::ErrorKind::InvalidData, e)))?;

    Ok((authority, auction_house, auction_house_data))
}

pub fn delegate(
    context: &mut ProgramTestContext,
    auction_house: &Pubkey,
    authority: &Keypair,
    auctioneer_authority: &Pubkey,
    auctioneer: &Pubkey,
) -> Transaction {
    let data = auction_house::instruction::DelegateAuctioneer {};

    let accounts = auction_house::accounts::DelegateAuctioneer {
        auction_house: *auction_house,
        authority: authority.pubkey(),
        auctioneer_authority: *auctioneer_authority,
        auctioneer: *auctioneer,
        system_program: system_program::id(),
    };

    let ix = Instruction {
        program_id: auction_house::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[authority],
        context.last_blockhash,
    )
}

pub async fn airdrop(
    context: &mut ProgramTestContext,
    receiver: &Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            receiver,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    Ok(())
}
