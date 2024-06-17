#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]
pub mod authorize;
pub mod bid;
pub mod cancel;
pub mod constants;
pub mod deposit;
pub mod errors;
pub mod execute_sale;
pub mod pda;
pub mod sell;
pub mod utils;
pub mod withdraw;

use crate::{authorize::*, bid::*, cancel::*, deposit::*, execute_sale::*, sell::*, withdraw::*};

use anchor_lang::prelude::*;

declare_id!("9sCGJFSVb7zyXfozXXiVyemaaNtbHVEiRy81HmQzGWG9");

#[program]
pub mod auctioneer {

    use super::*;

    pub fn authorize(ctx: Context<AuctioneerAuthorize>) -> Result<()> {
        auctioneer_authorize(ctx)
    }

    pub fn sell(
        ctx: Context<AuctioneerSell>,
        trade_state_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        auctioneer_authority_bump: u8,
        token_size: u64,
        start_time: i64,
        end_time: i64,
        reserve_price: Option<u64>,
        min_bid_increment: Option<u64>,
        time_ext_period: Option<u32>,
        time_ext_delta: Option<u32>,
    ) -> Result<()> {
        auctioneer_sell(
            ctx,
            trade_state_bump,
            free_trade_state_bump,
            program_as_signer_bump,
            auctioneer_authority_bump,
            token_size,
            start_time,
            end_time,
            reserve_price,
            min_bid_increment,
            time_ext_period,
            time_ext_delta,
        )
    }

    pub fn deposit(
        ctx: Context<AuctioneerDeposit>,
        escrow_payment_bump: u8,
        auctioneer_authority_bump: u8,
        amount: u64,
    ) -> Result<()> {
        auctioneer_deposit(ctx, escrow_payment_bump, auctioneer_authority_bump, amount)
    }

    pub fn buy(
        ctx: Context<AuctioneerBuy>,
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        auctioneer_authority_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> Result<()> {
        auctioneer_buy(
            ctx,
            trade_state_bump,
            escrow_payment_bump,
            auctioneer_authority_bump,
            buyer_price,
            token_size,
        )
    }

    #[inline(never)]
    pub fn execute_sale<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerExecuteSale<'info>>,
        escrow_payment_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        auctioneer_authority_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> Result<()> {
        auctioneer_execute_sale(
            ctx,
            escrow_payment_bump,
            free_trade_state_bump,
            program_as_signer_bump,
            auctioneer_authority_bump,
            buyer_price,
            token_size,
        )
    }

    pub fn withdraw(
        ctx: Context<AuctioneerWithdraw>,
        escrow_payment_bump: u8,
        auctioneer_authority_bump: u8,
        amount: u64,
    ) -> Result<()> {
        auctioneer_withdraw(ctx, escrow_payment_bump, auctioneer_authority_bump, amount)
    }

    pub fn cancel(
        ctx: Context<AuctioneerCancel>,
        auctioneer_authority_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> Result<()> {
        auctioneer_cancel(ctx, auctioneer_authority_bump, buyer_price, token_size)
    }
}
