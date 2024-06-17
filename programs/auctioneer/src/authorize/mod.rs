use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::errors::*;

use auction_house::{
    self,
    constants::{AUCTIONEER, PREFIX},
    AuctionHouse,
};

#[derive(Accounts, Clone)]
pub struct AuctioneerAuthorize<'info> {
    #[account(mut)]
    pub wallet: Signer<'info>,

    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = auction_house::id(),
        bump = auction_house.bump
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    #[account(
        init,
        payer = wallet,
        space = 8 + 1,
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref()
        ],
        bump
    )]
    pub auctioneer_authority: Account<'info, AuctioneerAuthority>,

    pub system_program: Program<'info, System>,
}

pub fn auctioneer_authorize(ctx: Context<AuctioneerAuthorize>) -> Result<()> {
    if ctx.accounts.wallet.key() != ctx.accounts.auction_house.authority {
        return err!(AuctioneerError::SignerNotAuth);
    }

    ctx.accounts.auctioneer_authority.bump = *ctx
        .bumps
        .get("auctioneer_authority")
        .ok_or(AuctioneerError::BumpSeedNotInHashMap)?;

    Ok(())
}

#[account]
pub struct AuctioneerAuthority {
    pub bump: u8,
}
