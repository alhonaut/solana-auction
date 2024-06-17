use anchor_lang::{prelude::*, solana_program::program::invoke, AnchorDeserialize};
use spl_token::instruction::approve;

use crate::{constants::*, errors::*, utils::*, AuctionHouse, *};

#[derive(Accounts, Clone)]
#[instruction(
    trade_state_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    token_size: u64
)]
pub struct AuctioneerSell<'info> {
    #[account(mut)]
    pub wallet: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,
    pub metadata: UncheckedAccount<'info>,
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
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            wallet.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &u64::MAX.to_le_bytes(),
            &token_size.to_le_bytes()
        ],
        bump
    )]
    pub seller_trade_state: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            wallet.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &0u64.to_le_bytes(),
            &token_size.to_le_bytes()
        ],
        bump
    )]
    pub free_seller_trade_state: UncheckedAccount<'info>,
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            auctioneer_authority.key().as_ref()
        ],
        bump
    )]
    pub auctioneer: Account<'info, Auctioneer>,
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], bump)]
    pub program_as_signer: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn auctioneer_sell<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerSell<'info>>,
    trade_state_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    token_size: u64,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;

    if !auction_house.has_auctioneer {
        return Err(AuctionHouseError::NoAuctioneerProgramSet.into());
    }

    let trade_state_canonical_bump = *ctx
        .bumps
        .get("seller_trade_state")
        .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;
    let free_trade_state_canonical_bump = *ctx
        .bumps
        .get("free_seller_trade_state")
        .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;
    let program_as_signer_canonical_bump = *ctx
        .bumps
        .get("program_as_signer")
        .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;

    if (trade_state_canonical_bump != trade_state_bump)
        || (free_trade_state_canonical_bump != free_trade_state_bump)
        || (program_as_signer_canonical_bump != program_as_signer_bump)
    {
        return Err(AuctionHouseError::BumpSeedNotInHashMap.into());
    }

    let mut accounts: AuctioneerSell<'info> = (*ctx.accounts).clone();

    sell_logic(
        &mut accounts,
        ctx.program_id,
        trade_state_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        u64::MAX,
        token_size,
    )
}

#[allow(clippy::extra_unused_lifetimes)]
fn sell_logic<'c, 'info>(
    accounts: &mut AuctioneerSell<'info>,
    program_id: &Pubkey,
    trade_state_bump: u8,
    _free_trade_state_bump: u8,
    _program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let wallet = &accounts.wallet;
    let token_account = &accounts.token_account;
    let metadata = &accounts.metadata;
    let authority = &accounts.authority;
    let seller_trade_state = &accounts.seller_trade_state;
    let free_seller_trade_state = &accounts.free_seller_trade_state;
    let auction_house = &accounts.auction_house;
    let auction_house_fee_account = &accounts.auction_house_fee_account;
    let token_program = &accounts.token_program;
    let system_program = &accounts.system_program;
    let program_as_signer = &accounts.program_as_signer;
    let rent = &accounts.rent;

    if !wallet.to_account_info().is_signer
        && (buyer_price == 0
            || free_seller_trade_state.data_is_empty()
            || !authority.to_account_info().is_signer
            || !auction_house.can_change_sale_price)
    {
        return Err(AuctionHouseError::SaleRequiresSigner.into());
    }

    let auction_house_key = auction_house.key();

    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];

    let (fee_payer, fee_seeds) = get_fee_payer(
        authority,
        wallet.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;
    assert_is_ata(
        &token_account.to_account_info(),
        &wallet.key(),
        &token_account.mint,
    )?;

    assert_metadata_valid(metadata, token_account)?;

    if token_size > token_account.amount {
        return Err(AuctionHouseError::InvalidTokenAmount.into());
    }

    if wallet.is_signer {
        invoke(
            &approve(
                &token_program.key(),
                &token_account.key(),
                &program_as_signer.key(),
                &wallet.key(),
                &[],
                token_size,
            )
            .unwrap(),
            &[
                token_program.to_account_info(),
                token_account.to_account_info(),
                program_as_signer.to_account_info(),
                wallet.to_account_info(),
            ],
        )?;
    }

    let ts_info = seller_trade_state.to_account_info();
    if ts_info.data_is_empty() {
        let token_account_key = token_account.key();
        let wallet_key = wallet.key();
        let ts_seeds = [
            PREFIX.as_bytes(),
            wallet_key.as_ref(),
            auction_house_key.as_ref(),
            token_account_key.as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &buyer_price.to_le_bytes(),
            &token_size.to_le_bytes(),
            &[trade_state_bump],
        ];
        create_or_allocate_account_raw(
            *program_id,
            &ts_info,
            &rent.to_account_info(),
            system_program,
            &fee_payer,
            TRADE_STATE_SIZE,
            fee_seeds,
            &ts_seeds,
        )?;
    }

    let data = &mut ts_info.data.borrow_mut();
    data[0] = trade_state_bump;

    Ok(())
}
