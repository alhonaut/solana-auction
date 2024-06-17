use anchor_lang::{
    prelude::*, solana_program::program::invoke_signed, AnchorDeserialize, InstructionData,
};
use anchor_spl::token::{Mint, Token, TokenAccount};

use auction_house::{
    self,
    constants::{AUCTIONEER, FEE_PAYER, PREFIX},
    cpi::accounts::AuctioneerCancel as AHCancel,
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse,
};

use crate::{constants::*, errors::*, sell::config::*};

#[derive(Accounts, Clone)]
#[instruction(
    auctioneer_authority_bump: u8,
    buyer_price: u64,
    token_size: u64
)]
pub struct AuctioneerCancel<'info> {
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    #[account(
        mut,
        seeds = [
            LISTING_CONFIG.as_bytes(),
            seller.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &token_size.to_le_bytes()
        ],
        bump,
    )]
    pub listing_config: Account<'info, ListingConfig>,
    pub seller: UncheckedAccount<'info>,
    #[account(mut)]
    pub wallet: Signer<'info>,
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,
    pub token_mint: Box<Account<'info, Mint>>,
    pub authority: UncheckedAccount<'info>,
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.bump,
        has_one = authority,
        has_one = auction_house_fee_account
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            FEE_PAYER.as_bytes()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.fee_payer_bump
    )]
    pub auction_house_fee_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub trade_state: UncheckedAccount<'info>,
    pub auctioneer_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            auctioneer_authority.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auctioneer.bump,
    )]
    pub auctioneer: Account<'info, auction_house::Auctioneer>,
    pub token_program: Program<'info, Token>,
}

pub fn auctioneer_cancel(
    ctx: Context<AuctioneerCancel>,
    auctioneer_authority_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    if ctx.accounts.trade_state.key() == ctx.accounts.listing_config.highest_bid.buyer_trade_state {
        return err!(AuctioneerError::CannotCancelHighestBid);
    }

    let cpi_program = ctx.accounts.auction_house_program.to_account_info();
    let cpi_accounts = AHCancel {
        wallet: ctx.accounts.wallet.to_account_info(),
        token_account: ctx.accounts.token_account.to_account_info(),
        token_mint: ctx.accounts.token_mint.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        trade_state: ctx.accounts.trade_state.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auctioneer_authority: ctx.accounts.auctioneer_authority.to_account_info(),
        auctioneer: ctx.accounts.auctioneer.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cancel_data = auction_house::instruction::AuctioneerCancel {
        buyer_price,
        token_size,
    };

    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: cpi_program.key(),
        accounts: cpi_accounts
            .to_account_metas(None)
            .into_iter()
            .zip(cpi_accounts.to_account_infos())
            .map(|mut pair| {
                pair.0.is_signer = pair.1.is_signer;
                if pair.0.pubkey == ctx.accounts.auctioneer_authority.key() {
                    pair.0.is_signer = true;
                }
                pair.0
            })
            .collect(),
        data: cancel_data.data(),
    };

    let auction_house = &ctx.accounts.auction_house;
    let ah_key = auction_house.key();
    let auctioneer_authority = &ctx.accounts.auctioneer_authority;
    let _aa_key = auctioneer_authority.key();

    let auctioneer_seeds = [
        AUCTIONEER.as_bytes(),
        ah_key.as_ref(),
        &[auctioneer_authority_bump],
    ];

    invoke_signed(&ix, &cpi_accounts.to_account_infos(), &[&auctioneer_seeds])?;

    if ctx.accounts.token_account.owner == ctx.accounts.wallet.key()
        && ctx.accounts.wallet.is_signer
    {
        let listing_config = &ctx.accounts.listing_config.to_account_info();
        let seller = &ctx.accounts.seller.to_account_info();

        let listing_config_lamports = listing_config.lamports();
        **seller.lamports.borrow_mut() = seller
            .lamports()
            .checked_add(listing_config_lamports)
            .unwrap();
        **listing_config.lamports.borrow_mut() = 0;

        let mut source_data = listing_config.data.borrow_mut();
        source_data.fill(0);
    }

    Ok(())
}
