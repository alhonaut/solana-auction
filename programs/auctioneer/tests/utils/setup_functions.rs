#![allow(dead_code)]

use anchor_client::solana_sdk::transaction::{Transaction, TransactionError};
use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use anchor_lang::prelude::AccountMeta;
use anchor_lang::solana_program::instruction::{Instruction, InstructionError};
use anchor_lang::solana_program::{system_instruction, system_program, sysvar};
use anchor_lang::{prelude::Pubkey, AccountDeserialize};
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::{self, get_associated_token_address};
use anchor_spl::token::spl_token;
use auction_house::pda::*;
use auction_house::AuctionHouse;
use auctioneer::pda::*;
use nft_minter::pda::*;
use nft_minter::utils::{token_metadata_program_id, Creator};
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use std::io;

// Error = Error code
pub const ERR_ACCOUNT_NOT_INITIALIZED: u32 = 3012;

pub const ONE_SOL: u64 = 1_000_000_000;

pub fn assert_error(error: BanksClientError, expected_error: u32) {
    match error {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::Custom(e),
        )) => assert_eq!(e, expected_error),
        _ => assert!(false),
    }
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

#[derive(Debug)]
pub struct NFT {
    pub mint: Keypair,          // Mint
    pub owner: Keypair,         // Seller
    pub ata: Pubkey,            // Token account
    pub metadata: Pubkey,       // Metaplex Metadata
    pub master_edition: Pubkey, // Metaplex Master Edition
}

pub async fn create_nft(
    context: &mut ProgramTestContext,
    metadata_creators: Option<Vec<Creator>>,
) -> Result<NFT, BanksClientError> {
    let mint = Keypair::new();
    let owner = Keypair::new();
    airdrop(context, &owner.pubkey(), 10 * ONE_SOL)
        .await
        .unwrap();

    let ata = get_associated_token_address(&owner.pubkey(), &mint.pubkey());
    let (metadata, _) = find_metadata_account(&mint.pubkey());
    let (master_edition, _) = find_master_edition_account(&mint.pubkey());

    // CreateToken
    let create_token_ix = Instruction {
        program_id: nft_minter::id(),
        data: nft_minter::instruction::CreateToken {
            name: "Solana Course NFT".to_string(),
            symbol: "SOLC".to_string(),
            uri: "https://raw.githubusercontent.com/arsenijkovalov/nft-assets/main/assets/nft.json"
                .to_string(),
            creators: metadata_creators,
            seller_fee_basis_points: 10,
            is_mutable: false,
        }
        .data(),
        accounts: nft_minter::accounts::CreateToken {
            payer: owner.pubkey(),
            mint_account: mint.pubkey(),
            mint_authority: owner.pubkey(),
            update_authority: owner.pubkey(),
            metadata_account: metadata,
            token_metadata_program: token_metadata_program_id(),
            system_program: system_program::id(),
            token_program: anchor_spl::token::ID,
            rent: sysvar::rent::id(),
        }
        .to_account_metas(None),
    };

    // MintToken
    let mint_token_ix = Instruction {
        program_id: nft_minter::id(),
        data: nft_minter::instruction::MintToken {
            max_supply: Some(0),
        }
        .data(),
        accounts: nft_minter::accounts::MintToken {
            payer: owner.pubkey(),
            mint_account: mint.pubkey(),
            mint_authority: owner.pubkey(),
            update_authority: owner.pubkey(),
            associated_token_account: ata,
            metadata_account: metadata,
            edition_account: master_edition,
            token_metadata_program: token_metadata_program_id(),
            system_program: system_program::id(),
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            rent: sysvar::rent::id(),
        }
        .to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_token_ix, mint_token_ix],
        Some(&owner.pubkey()),
        &[&mint, &owner],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await?;

    Ok(NFT {
        mint,
        owner,
        ata,
        metadata,
        master_edition,
    })
}

pub fn auctioneer_program_test() -> ProgramTest {
    let mut program = ProgramTest::new("auctioneer", auctioneer::id(), None);
    program.add_program("auction_house", auction_house::id(), None);
    program.add_program("nft_minter", nft_minter::id(), None);
    program.add_program("mpl_token_metadata", token_metadata_program_id(), None);
    program
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

    // DelegateAuctioneer
    let (auctioneer_authority, _) = find_auctioneer_authority_address(&auction_house);
    let (auctioneer, _) = find_auctioneer_address(&auction_house, &auctioneer_authority);

    let delegate_auctioneer_ix = Instruction {
        program_id: auction_house::id(),
        data: auction_house::instruction::DelegateAuctioneer {}.data(),
        accounts: auction_house::accounts::DelegateAuctioneer {
            auction_house,
            authority: authority.pubkey(),
            auctioneer_authority,
            auctioneer,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
    };

    // AuctioneerAuthorize
    let auctioneer_authorize_ix = Instruction {
        program_id: auctioneer::id(),
        data: auctioneer::instruction::Authorize {}.data(),
        accounts: auctioneer::accounts::AuctioneerAuthorize {
            wallet: authority.pubkey(),
            auction_house,
            auctioneer_authority,
            system_program: system_program::id(),
        }
        .to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[
            create_auction_house_ix,
            delegate_auctioneer_ix,
            auctioneer_authorize_ix,
        ],
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

pub fn sell(
    context: &mut ProgramTestContext,
    auction_house: &Pubkey,
    auction_house_data: &AuctionHouse,
    token: &NFT,
    start_time: i64,
    end_time: i64,
    reserve_price: Option<u64>,
    min_bid_increment: Option<u64>,
    time_ext_period: Option<u32>,
    time_ext_delta: Option<u32>,
) -> (auctioneer::accounts::AuctioneerSell, Transaction) {
    let (seller_trade_state, seller_trade_state_bump) = find_auctioneer_trade_state_address(
        &token.owner.pubkey(),
        auction_house,
        &token.ata,
        &auction_house_data.treasury_mint,
        &token.mint.pubkey(),
        1,
    );

    let (free_seller_trade_state, free_seller_trade_state_bump) = find_trade_state_address(
        &token.owner.pubkey(),
        auction_house,
        &token.ata,
        &auction_house_data.treasury_mint,
        &token.mint.pubkey(),
        0,
        1,
    );

    let (listing_config, _) = find_listing_config_address(
        &token.owner.pubkey(),
        auction_house,
        &token.ata,
        &auction_house_data.treasury_mint,
        &token.mint.pubkey(),
        1,
    );

    let (program_as_signer, program_as_signer_bump) = find_program_as_signer_address();

    let (auctioneer_authority, auctioneer_authority_bump) =
        find_auctioneer_authority_address(auction_house);

    let (auctioneer, _) = find_auctioneer_address(auction_house, &auctioneer_authority);

    let data = auctioneer::instruction::Sell {
        trade_state_bump: seller_trade_state_bump,
        free_trade_state_bump: free_seller_trade_state_bump,
        program_as_signer_bump,
        auctioneer_authority_bump,
        token_size: 1,
        start_time,
        end_time,
        reserve_price,
        min_bid_increment,
        time_ext_period,
        time_ext_delta,
    };

    let accounts = auctioneer::accounts::AuctioneerSell {
        auction_house_program: auction_house::id(),
        listing_config,
        wallet: token.owner.pubkey(),
        token_account: token.ata,
        metadata: token.metadata,
        authority: auction_house_data.authority,
        auction_house: *auction_house,
        auction_house_fee_account: auction_house_data.auction_house_fee_account,
        seller_trade_state,
        free_seller_trade_state,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        program_as_signer,
        rent: sysvar::rent::id(),
        auctioneer_authority,
        auctioneer,
    };

    let ix = Instruction {
        program_id: auctioneer::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    (
        accounts,
        Transaction::new_signed_with_payer(
            &[ix],
            Some(&token.owner.pubkey()),
            &[&token.owner],
            context.last_blockhash,
        ),
    )
}

pub fn buy(
    context: &mut ProgramTestContext,
    auction_house: &Pubkey,
    auction_house_data: &AuctionHouse,
    token: &NFT,
    buyer: &Keypair,
    buyer_price: u64, // Bid amount
) -> (auctioneer::accounts::AuctioneerBuy, Transaction) {
    let (listing_config, _) = find_listing_config_address(
        &token.owner.pubkey(),
        auction_house,
        &token.ata,
        &auction_house_data.treasury_mint,
        &token.mint.pubkey(),
        1,
    );
    let (escrow_payment_account, escrow_payment_account_bump) =
        find_escrow_payment_account_address(auction_house, &buyer.pubkey());
    let (auctioneer_authority, auctioneer_authority_bump) =
        find_auctioneer_authority_address(auction_house);
    let (auctioneer, _) = find_auctioneer_address(auction_house, &auctioneer_authority);
    let (buyer_trade_state, buyer_trade_state_bump) = find_trade_state_address(
        &buyer.pubkey(),
        auction_house,
        &token.ata,
        &auction_house_data.treasury_mint,
        &token.mint.pubkey(),
        buyer_price,
        1,
    );

    let data = auctioneer::instruction::Buy {
        trade_state_bump: buyer_trade_state_bump,
        escrow_payment_bump: escrow_payment_account_bump,
        auctioneer_authority_bump: auctioneer_authority_bump,
        token_size: 1,
        buyer_price,
    };

    let accounts = auctioneer::accounts::AuctioneerBuy {
        auction_house_program: auction_house::id(),
        listing_config,
        seller: token.owner.pubkey(),
        wallet: buyer.pubkey(),
        token_account: token.ata,
        metadata: token.metadata,
        authority: auction_house_data.authority,
        auction_house: *auction_house,
        auction_house_fee_account: auction_house_data.auction_house_fee_account,
        buyer_trade_state,
        token_program: spl_token::id(),
        treasury_mint: auction_house_data.treasury_mint,
        payment_account: buyer.pubkey(),
        transfer_authority: buyer.pubkey(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
        escrow_payment_account,
        auctioneer_authority,
        auctioneer,
    };

    let ix = Instruction {
        program_id: auctioneer::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    (
        accounts,
        Transaction::new_signed_with_payer(
            &[ix],
            Some(&buyer.pubkey()),
            &[buyer],
            context.last_blockhash,
        ),
    )
}

pub async fn execute_sale(
    context: &mut ProgramTestContext,
    auction_house: &Pubkey,
    auction_house_data: &AuctionHouse,
    token: &NFT,
    metadata_creators: Option<Vec<Creator>>,
    sell_accounts: &auctioneer::accounts::AuctioneerSell,
    buy_accounts: &auctioneer::accounts::AuctioneerBuy,
    highest_bid: u64,
    signer_payer: &Keypair,
) -> (auctioneer::accounts::AuctioneerExecuteSale, Transaction) {
    if signer_payer.pubkey().eq(&auction_house_data.authority) {
        airdrop(
            context,
            &auction_house_data.auction_house_fee_account,
            10 * ONE_SOL,
        )
        .await
        .expect("Failed to airdrop SOLs to Auction House fee account");
    }

    let (auctioneer_authority, auctioneer_authority_bump) =
        find_auctioneer_authority_address(auction_house);
    let (auctioneer, _) = find_auctioneer_address(auction_house, &auctioneer_authority);
    let buyer_receipt_token_account =
        get_associated_token_address(&buy_accounts.wallet, &token.mint.pubkey());
    let (_, escrow_payment_account_bump) =
        find_escrow_payment_account_address(&auction_house, &buy_accounts.wallet);
    let (_, program_as_signer_bump) = find_program_as_signer_address();

    let (_, free_seller_trade_state_bump) = find_trade_state_address(
        &token.owner.pubkey(),
        auction_house,
        &sell_accounts.token_account,
        &auction_house_data.treasury_mint,
        &token.mint.pubkey(),
        0,
        1,
    );

    let data = auctioneer::instruction::ExecuteSale {
        escrow_payment_bump: escrow_payment_account_bump,
        free_trade_state_bump: free_seller_trade_state_bump,
        program_as_signer_bump,
        auctioneer_authority_bump: auctioneer_authority_bump,
        token_size: 1,
        buyer_price: highest_bid,
    };

    let accounts = auctioneer::accounts::AuctioneerExecuteSale {
        auction_house_program: auction_house::id(),
        listing_config: sell_accounts.listing_config,
        buyer: buy_accounts.wallet,
        seller: sell_accounts.wallet,
        authority: auction_house_data.authority,
        auction_house: *auction_house,
        metadata: token.metadata,
        token_account: sell_accounts.token_account,
        seller_trade_state: sell_accounts.seller_trade_state,
        buyer_trade_state: buy_accounts.buyer_trade_state,
        token_program: spl_token::id(),
        free_trade_state: sell_accounts.free_seller_trade_state,
        seller_payment_receipt_account: token.owner.pubkey(),
        buyer_receipt_token_account,
        escrow_payment_account: buy_accounts.escrow_payment_account,
        token_mint: token.mint.pubkey(),
        auction_house_fee_account: auction_house_data.auction_house_fee_account,
        auction_house_treasury: auction_house_data.auction_house_treasury,
        treasury_mint: auction_house_data.treasury_mint,
        program_as_signer: sell_accounts.program_as_signer,
        system_program: system_program::id(),
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::id(),
        auctioneer_authority,
        auctioneer,
    };

    let mut account_metas = accounts.to_account_metas(None);
    if let Some(creators) = metadata_creators {
        for creator in &creators {
            account_metas.push(AccountMeta {
                pubkey: creator.address,
                is_signer: false,
                is_writable: true,
            });
        }
    }

    let ix = Instruction {
        program_id: auctioneer::id(),
        data: data.data(),
        accounts: account_metas,
    };

    (
        accounts,
        Transaction::new_signed_with_payer(
            &[ix],
            Some(&signer_payer.pubkey()),
            &[signer_payer],
            context.last_blockhash,
        ),
    )
}

pub fn deposit(
    context: &mut ProgramTestContext,
    auction_house: &Pubkey,
    auction_house_data: &AuctionHouse,
    buyer: &Keypair,
    amount: u64,
) -> (auctioneer::accounts::AuctioneerDeposit, Transaction) {
    let (escrow_payment_account, escrow_payment_account_bump) =
        find_escrow_payment_account_address(auction_house, &buyer.pubkey());
    let (auctioneer_authority, auctioneer_authority_bump) =
        find_auctioneer_authority_address(auction_house);
    let (auctioneer, _) = find_auctioneer_address(auction_house, &auctioneer_authority);

    let data = auctioneer::instruction::Deposit {
        amount,
        escrow_payment_bump: escrow_payment_account_bump,
        auctioneer_authority_bump: auctioneer_authority_bump,
    };

    let accounts = auctioneer::accounts::AuctioneerDeposit {
        auction_house_program: auction_house::id(),
        wallet: buyer.pubkey(),
        authority: auction_house_data.authority,
        auction_house: *auction_house,
        auction_house_fee_account: auction_house_data.auction_house_fee_account,
        token_program: spl_token::id(),
        treasury_mint: auction_house_data.treasury_mint,
        payment_account: buyer.pubkey(),
        transfer_authority: buyer.pubkey(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
        escrow_payment_account,
        auctioneer_authority,
        auctioneer,
    };

    let ix = Instruction {
        program_id: auctioneer::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    (
        accounts,
        Transaction::new_signed_with_payer(
            &[ix],
            Some(&buyer.pubkey()),
            &[buyer],
            context.last_blockhash,
        ),
    )
}

pub fn withdraw(
    context: &mut ProgramTestContext,
    auction_house: &Pubkey,
    auction_house_data: &AuctionHouse,
    buyer: &Keypair,
    amount: u64,
) -> (auctioneer::accounts::AuctioneerWithdraw, Transaction) {
    let (escrow_payment_account, escrow_payment_account_bump) =
        find_escrow_payment_account_address(auction_house, &buyer.pubkey());
    let (auctioneer_authority, auctioneer_authority_bump) =
        find_auctioneer_authority_address(auction_house);
    let (auctioneer, _) = find_auctioneer_address(auction_house, &auctioneer_authority);

    let data = auctioneer::instruction::Withdraw {
        escrow_payment_bump: escrow_payment_account_bump,
        auctioneer_authority_bump,
        amount,
    };

    let accounts = auctioneer::accounts::AuctioneerWithdraw {
        auction_house_program: auction_house::id(),
        wallet: buyer.pubkey(),
        escrow_payment_account,
        receipt_account: buyer.pubkey(),
        treasury_mint: auction_house_data.treasury_mint,
        authority: auction_house_data.authority,
        auction_house: *auction_house,
        auction_house_fee_account: auction_house_data.auction_house_fee_account,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        associated_token_program: associated_token::ID,
        rent: sysvar::rent::id(),
        auctioneer_authority,
        auctioneer,
    };

    let ix = Instruction {
        program_id: auctioneer::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    (
        accounts,
        Transaction::new_signed_with_payer(
            &[ix],
            Some(&buyer.pubkey()),
            &[buyer],
            context.last_blockhash,
        ),
    )
}

fn cancel(
    context: &mut ProgramTestContext,
    auction_house: &Pubkey,
    auction_house_data: &AuctionHouse,
    token: &NFT,
    wallet: &Keypair,
    trade_state: &Pubkey,
    buyer_price: u64,
) -> (auctioneer::accounts::AuctioneerCancel, Transaction) {
    let (listing_config, _) = find_listing_config_address(
        &token.owner.pubkey(),
        auction_house,
        &token.ata,
        &auction_house_data.treasury_mint,
        &token.mint.pubkey(),
        1,
    );
    let (auctioneer_authority, auctioneer_authority_bump) =
        find_auctioneer_authority_address(auction_house);
    let (auctioneer, _) = find_auctioneer_address(auction_house, &auctioneer_authority);

    let data = auctioneer::instruction::Cancel {
        auctioneer_authority_bump,
        buyer_price,
        token_size: 1,
    };

    let accounts = auctioneer::accounts::AuctioneerCancel {
        auction_house_program: auction_house::id(),
        listing_config,
        seller: token.owner.pubkey(),
        wallet: wallet.pubkey(),
        token_account: token.ata,
        token_mint: token.mint.pubkey(),
        authority: auction_house_data.authority,
        auction_house: *auction_house,
        auction_house_fee_account: auction_house_data.auction_house_fee_account,
        trade_state: *trade_state,
        auctioneer_authority,
        auctioneer,
        token_program: spl_token::id(),
    };

    let ix = Instruction {
        program_id: auctioneer::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    (
        accounts,
        Transaction::new_signed_with_payer(
            &[ix],
            Some(&wallet.pubkey()),
            &[wallet],
            context.last_blockhash,
        ),
    )
}

pub fn cancel_sell(
    context: &mut ProgramTestContext,
    auction_house: &Pubkey,
    auction_house_data: &AuctionHouse,
    token: &NFT,
    seller_trade_state: &Pubkey,
) -> (auctioneer::accounts::AuctioneerCancel, Transaction) {
    cancel(
        context,
        auction_house,
        auction_house_data,
        token,
        &token.owner,
        seller_trade_state,
        u64::MAX,
    )
}

pub fn cancel_buy(
    context: &mut ProgramTestContext,
    auction_house: &Pubkey,
    auction_house_data: &AuctionHouse,
    token: &NFT,
    buyer: &Keypair,
    buyer_trade_state: &Pubkey,
    buyer_price: u64, // Bid amount
) -> (auctioneer::accounts::AuctioneerCancel, Transaction) {
    cancel(
        context,
        auction_house,
        auction_house_data,
        token,
        buyer,
        buyer_trade_state,
        buyer_price,
    )
}

pub fn close_escrow_account(
    context: &mut ProgramTestContext,
    auction_house: &Pubkey,
    buyer: &Keypair,
) -> (auction_house::accounts::CloseEscrowAccount, Transaction) {
    let (escrow_payment_account, escrow_payment_account_bump) =
        find_escrow_payment_account_address(auction_house, &buyer.pubkey());

    let data = auction_house::instruction::CloseEscrowAccount {
        escrow_payment_bump: escrow_payment_account_bump,
    };

    let accounts = auction_house::accounts::CloseEscrowAccount {
        wallet: buyer.pubkey(),
        escrow_payment_account,
        auction_house: *auction_house,
        system_program: system_program::id(),
    };

    let ix = Instruction {
        program_id: auction_house::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    (
        accounts,
        Transaction::new_signed_with_payer(
            &[ix],
            Some(&buyer.pubkey()),
            &[buyer],
            context.last_blockhash,
        ),
    )
}

pub fn withdraw_from_fee(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    auction_house: &Pubkey,
    auction_house_data: &AuctionHouse,
    amount: u64,
) -> (auction_house::accounts::WithdrawFromFee, Transaction) {
    let data = auction_house::instruction::WithdrawFromFee { amount };

    let accounts = auction_house::accounts::WithdrawFromFee {
        authority: auction_house_data.authority,
        fee_withdrawal_destination: auction_house_data.fee_withdrawal_destination,
        auction_house_fee_account: auction_house_data.auction_house_fee_account,
        auction_house: *auction_house,
        system_program: system_program::id(),
    };

    let ix = Instruction {
        program_id: auction_house::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    (
        accounts,
        Transaction::new_signed_with_payer(
            &[ix],
            Some(&authority.pubkey()),
            &[authority],
            context.last_blockhash,
        ),
    )
}

pub fn withdraw_from_treasury(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    auction_house: &Pubkey,
    auction_house_data: &AuctionHouse,
    amount: u64,
) -> (auction_house::accounts::WithdrawFromTreasury, Transaction) {
    let data = auction_house::instruction::WithdrawFromTreasury { amount };

    let accounts = auction_house::accounts::WithdrawFromTreasury {
        treasury_mint: auction_house_data.treasury_mint,
        treasury_withdrawal_destination: auction_house_data.treasury_withdrawal_destination,
        auction_house_treasury: auction_house_data.auction_house_treasury,
        authority: auction_house_data.authority,
        auction_house: *auction_house,
        token_program: spl_token::id(),
        system_program: system_program::id(),
    };

    let ix = Instruction {
        program_id: auction_house::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    (
        accounts,
        Transaction::new_signed_with_payer(
            &[ix],
            Some(&authority.pubkey()),
            &[authority],
            context.last_blockhash,
        ),
    )
}
