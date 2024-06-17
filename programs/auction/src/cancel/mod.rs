use crate::{constants::*, errors::*, utils::*, AuctionHouse, *};
use anchor_lang::solana_program::program_memory::sol_memset;
use anchor_lang::{prelude::*, solana_program::program::invoke, AnchorDeserialize};
use spl_token::instruction::revoke;

#[derive(Accounts, Clone)]
#[instruction(buyer_price: u64, token_size: u64)]
pub struct AuctioneerCancel<'info> {
    #[account(mut)]
    pub wallet: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,
    pub token_mint: Box<Account<'info, Mint>>,
    pub authority: UncheckedAccount<'info>,
    pub auctioneer_authority: Signer<'info>,
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        bump=auction_house.bump,
        has_one=authority,
        has_one=auction_house_fee_account
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            FEE_PAYER.as_bytes()
        ],
        bump=auction_house.fee_payer_bump
    )]
    pub auction_house_fee_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub trade_state: UncheckedAccount<'info>,
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            auctioneer_authority.key().as_ref()
        ],
        bump = auctioneer.bump
    )]
    pub auctioneer: Account<'info, Auctioneer>,

    pub token_program: Program<'info, Token>,
}

pub fn auctioneer_cancel<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerCancel<'info>>,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;

    if !auction_house.has_auctioneer {
        return Err(AuctionHouseError::NoAuctioneerProgramSet.into());
    }

    let mut accounts: AuctioneerCancel<'info> = (*ctx.accounts).clone();

    cancel_logic(&mut accounts, buyer_price, token_size)
}

#[allow(clippy::needless_lifetimes)]
#[allow(clippy::extra_unused_lifetimes)]
fn cancel_logic<'c, 'info>(
    accounts: &mut AuctioneerCancel<'info>,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let wallet = &accounts.wallet;
    let token_account = &accounts.token_account;
    let token_mint = &accounts.token_mint;
    let authority = &accounts.authority;
    let auction_house = &accounts.auction_house;
    let auction_house_fee_account = &accounts.auction_house_fee_account;
    let trade_state = &accounts.trade_state;
    let token_program = &accounts.token_program;

    let ts_bump = trade_state.try_borrow_data()?[0];
    assert_valid_trade_state(
        &wallet.key(),
        auction_house,
        buyer_price,
        token_size,
        &trade_state.to_account_info(),
        &token_account.mint.key(),
        &token_account.key(),
        ts_bump,
    )?;
    assert_keys_equal(token_mint.key(), token_account.mint)?;
    if !wallet.to_account_info().is_signer && !authority.to_account_info().is_signer {
        return Err(AuctionHouseError::NoValidSignerPresent.into());
    }

    let auction_house_key = auction_house.key();
    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];

    let (fee_payer, _) = get_fee_payer(
        authority,
        wallet.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;

    if token_account.owner == wallet.key() && wallet.is_signer {
        invoke(
            &revoke(
                &token_program.key(),
                &token_account.key(),
                &wallet.key(),
                &[],
            )
            .unwrap(),
            &[
                token_program.to_account_info(),
                token_account.to_account_info(),
                wallet.to_account_info(),
            ],
        )?;
    }

    let curr_lamp = trade_state.lamports();
    **trade_state.lamports.borrow_mut() = 0;

    **fee_payer.lamports.borrow_mut() = fee_payer
        .lamports()
        .checked_add(curr_lamp)
        .ok_or(AuctionHouseError::NumericalOverflow)?;

    #[allow(clippy::explicit_auto_deref)]
    sol_memset(*trade_state.try_borrow_mut_data()?, 0, TRADE_STATE_SIZE);

    Ok(())
}
