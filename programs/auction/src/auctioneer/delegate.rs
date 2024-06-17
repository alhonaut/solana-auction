use anchor_lang::prelude::*;

use crate::{constants::*, errors::AuctionHouseError, AuctionHouse, Auctioneer};

#[derive(Accounts)]
pub struct DelegateAuctioneer<'info> {
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        bump = auction_house.bump,
        has_one = authority
    )]
    pub auction_house: Account<'info, AuctionHouse>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub auctioneer_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = authority,
        space = AUCTIONEER_SIZE,
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            auctioneer_authority.key().as_ref()
        ],
        bump
    )]
    pub auctioneer: Account<'info, Auctioneer>,

    pub system_program: Program<'info, System>,
}

pub fn delegate_auctioneer<'info>(
    ctx: Context<'_, '_, '_, 'info, DelegateAuctioneer<'info>>,
) -> Result<()> {
    let auction_house = &mut ctx.accounts.auction_house;

    if auction_house.has_auctioneer {
        return Err(AuctionHouseError::AuctionHouseAlreadyDelegated.into());
    }

    auction_house.has_auctioneer = true;
    auction_house.auctioneer_address = ctx.accounts.auctioneer.key();

    let auctioneer = &mut ctx.accounts.auctioneer;
    auctioneer.auctioneer_authority = ctx.accounts.auctioneer_authority.key();
    auctioneer.auction_house = ctx.accounts.auction_house.key();
    auctioneer.bump = *ctx
        .bumps
        .get("auctioneer")
        .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;

    Ok(())
}
