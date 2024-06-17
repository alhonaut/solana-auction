#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]
pub mod auctioneer;
pub mod bid;
pub mod cancel;
pub mod constants;
pub mod deposit;
pub mod errors;
pub mod execute_sale;
pub mod pda;
pub mod sell;
pub mod state;
pub mod utils;
pub mod withdraw;

pub use state::*;

use crate::{
    auctioneer::*, bid::*, cancel::*, constants::*, deposit::*, errors::AuctionHouseError,
    execute_sale::*, sell::*, utils::*, withdraw::*,
};

use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
    AnchorDeserialize, AnchorSerialize,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

anchor_lang::declare_id!("FMrPvDk4xZNykJ2aWCmyKCzQ12qhJ6SR9tS67fhLbx8x");

#[program]
pub mod auction_house {
    use super::*;

    pub fn withdraw_from_fee<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawFromFee<'info>>,
        amount: u64,
    ) -> Result<()> {
        let auction_house_fee_account = &ctx.accounts.auction_house_fee_account;
        let fee_withdrawal_destination = &ctx.accounts.fee_withdrawal_destination;
        let auction_house = &ctx.accounts.auction_house;
        let system_program = &ctx.accounts.system_program;

        let auction_house_key = auction_house.key();
        let seeds = [
            PREFIX.as_bytes(),
            auction_house_key.as_ref(),
            FEE_PAYER.as_bytes(),
            &[auction_house.fee_payer_bump],
        ];

        invoke_signed(
            &system_instruction::transfer(
                &auction_house_fee_account.key(),
                &fee_withdrawal_destination.key(),
                amount,
            ),
            &[
                auction_house_fee_account.to_account_info(),
                fee_withdrawal_destination.to_account_info(),
                system_program.to_account_info(),
            ],
            &[&seeds],
        )?;

        Ok(())
    }

    pub fn withdraw_from_treasury<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawFromTreasury<'info>>,
        amount: u64,
    ) -> Result<()> {
        let treasury_mint = &ctx.accounts.treasury_mint;
        let treasury_withdrawal_destination = &ctx.accounts.treasury_withdrawal_destination;
        let auction_house_treasury = &ctx.accounts.auction_house_treasury;
        let auction_house = &ctx.accounts.auction_house;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;

        let is_native = treasury_mint.key() == spl_token::native_mint::id();
        let auction_house_seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref(),
            &[auction_house.bump],
        ];

        let ah_key = auction_house.key();
        let auction_house_treasury_seeds = [
            PREFIX.as_bytes(),
            ah_key.as_ref(),
            TREASURY.as_bytes(),
            &[auction_house.treasury_bump],
        ];
        if !is_native {
            invoke_signed(
                &spl_token::instruction::transfer(
                    token_program.key,
                    &auction_house_treasury.key(),
                    &treasury_withdrawal_destination.key(),
                    &auction_house.key(),
                    &[],
                    amount,
                )?,
                &[
                    auction_house_treasury.to_account_info(),
                    treasury_withdrawal_destination.to_account_info(),
                    token_program.to_account_info(),
                    auction_house.to_account_info(),
                ],
                &[&auction_house_seeds],
            )?;
        } else {
            invoke_signed(
                &system_instruction::transfer(
                    &auction_house_treasury.key(),
                    &treasury_withdrawal_destination.key(),
                    amount,
                ),
                &[
                    auction_house_treasury.to_account_info(),
                    treasury_withdrawal_destination.to_account_info(),
                    system_program.to_account_info(),
                ],
                &[&auction_house_treasury_seeds],
            )?;
        }

        Ok(())
    }

    pub fn update_auction_house<'info>(
        ctx: Context<'_, '_, '_, 'info, UpdateAuctionHouse<'info>>,
        seller_fee_basis_points: Option<u16>,
        can_change_sale_price: Option<bool>,
    ) -> Result<()> {
        let treasury_mint = &ctx.accounts.treasury_mint;
        let payer = &ctx.accounts.payer;
        let new_authority = &ctx.accounts.new_authority;
        let auction_house = &mut ctx.accounts.auction_house;
        let fee_withdrawal_destination = &ctx.accounts.fee_withdrawal_destination;
        let treasury_withdrawal_destination_owner =
            &ctx.accounts.treasury_withdrawal_destination_owner;
        let treasury_withdrawal_destination = &ctx.accounts.treasury_withdrawal_destination;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;
        let associated_token_program = &ctx.accounts.associated_token_program;
        let rent = &ctx.accounts.rent;
        let is_native = treasury_mint.key() == spl_token::native_mint::id();

        if let Some(sfbp) = seller_fee_basis_points {
            if sfbp > 10000 {
                return Err(AuctionHouseError::InvalidBasisPoints.into());
            }

            auction_house.seller_fee_basis_points = sfbp;
        }

        if let Some(chsp) = can_change_sale_price {
            auction_house.can_change_sale_price = chsp;
        }

        auction_house.authority = new_authority.key();
        auction_house.treasury_withdrawal_destination = treasury_withdrawal_destination.key();
        auction_house.fee_withdrawal_destination = fee_withdrawal_destination.key();

        if !is_native {
            if treasury_withdrawal_destination.data_is_empty() {
                make_ata(
                    treasury_withdrawal_destination.to_account_info(),
                    treasury_withdrawal_destination_owner.to_account_info(),
                    treasury_mint.to_account_info(),
                    payer.to_account_info(),
                    associated_token_program.to_account_info(),
                    token_program.to_account_info(),
                    system_program.to_account_info(),
                    rent.to_account_info(),
                    &[],
                )?;
            }

            assert_is_ata(
                &treasury_withdrawal_destination.to_account_info(),
                &treasury_withdrawal_destination_owner.key(),
                &treasury_mint.key(),
            )?;
        } else {
            assert_keys_equal(
                treasury_withdrawal_destination.key(),
                treasury_withdrawal_destination_owner.key(),
            )?;
        }

        Ok(())
    }

    pub fn create_auction_house<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateAuctionHouse<'info>>,
        _bump: u8,
        fee_payer_bump: u8,
        treasury_bump: u8,
        seller_fee_basis_points: u16,
        can_change_sale_price: bool,
    ) -> Result<()> {
        let treasury_mint = &ctx.accounts.treasury_mint;
        let payer = &ctx.accounts.payer;
        let authority = &ctx.accounts.authority;
        let auction_house = &mut ctx.accounts.auction_house;
        let auction_house_fee_account = &ctx.accounts.auction_house_fee_account;
        let auction_house_treasury = &ctx.accounts.auction_house_treasury;
        let fee_withdrawal_destination = &ctx.accounts.fee_withdrawal_destination;
        let treasury_withdrawal_destination_owner =
            &ctx.accounts.treasury_withdrawal_destination_owner;
        let treasury_withdrawal_destination = &ctx.accounts.treasury_withdrawal_destination;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;
        let associated_token_program = &ctx.accounts.associated_token_program;
        let rent = &ctx.accounts.rent;

        auction_house.bump = *ctx
            .bumps
            .get("auction_house")
            .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;

        if fee_payer_bump
            != *ctx
                .bumps
                .get("auction_house_fee_account")
                .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?
        {
            return Err(AuctionHouseError::BumpSeedNotInHashMap.into());
        }
        auction_house.fee_payer_bump = fee_payer_bump;

        if treasury_bump
            != *ctx
                .bumps
                .get("auction_house_treasury")
                .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?
        {
            return Err(AuctionHouseError::BumpSeedNotInHashMap.into());
        }
        auction_house.treasury_bump = treasury_bump;

        if seller_fee_basis_points > 10000 {
            return Err(AuctionHouseError::InvalidBasisPoints.into());
        }
        auction_house.seller_fee_basis_points = seller_fee_basis_points;
        auction_house.can_change_sale_price = can_change_sale_price;
        auction_house.creator = authority.key();
        auction_house.authority = authority.key();
        auction_house.treasury_mint = treasury_mint.key();
        auction_house.auction_house_fee_account = auction_house_fee_account.key();
        auction_house.auction_house_treasury = auction_house_treasury.key();
        auction_house.treasury_withdrawal_destination = treasury_withdrawal_destination.key();
        auction_house.fee_withdrawal_destination = fee_withdrawal_destination.key();

        let is_native = treasury_mint.key() == spl_token::native_mint::id();

        let ah_key = auction_house.key();

        let auction_house_treasury_seeds = [
            PREFIX.as_bytes(),
            ah_key.as_ref(),
            TREASURY.as_bytes(),
            &[treasury_bump],
        ];

        create_program_token_account_if_not_present(
            auction_house_treasury,
            system_program,
            payer,
            token_program,
            treasury_mint,
            &auction_house.to_account_info(),
            rent,
            &auction_house_treasury_seeds,
            &[],
            is_native,
        )?;

        if !is_native {
            if treasury_withdrawal_destination.data_is_empty() {
                make_ata(
                    treasury_withdrawal_destination.to_account_info(),
                    treasury_withdrawal_destination_owner.to_account_info(),
                    treasury_mint.to_account_info(),
                    payer.to_account_info(),
                    associated_token_program.to_account_info(),
                    token_program.to_account_info(),
                    system_program.to_account_info(),
                    rent.to_account_info(),
                    &[],
                )?;
            }

            assert_is_ata(
                &treasury_withdrawal_destination.to_account_info(),
                &treasury_withdrawal_destination_owner.key(),
                &treasury_mint.key(),
            )?;
        } else {
            assert_keys_equal(
                treasury_withdrawal_destination.key(),
                treasury_withdrawal_destination_owner.key(),
            )?;
        }

        Ok(())
    }

    pub fn auctioneer_buy<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerBuy<'info>>,
        trade_state_bump: u8,
        escrow_payment_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> Result<()> {
        bid::auctioneer_private_bid(
            ctx,
            trade_state_bump,
            escrow_payment_bump,
            buyer_price,
            token_size,
        )
    }

    pub fn auctioneer_cancel<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerCancel<'info>>,
        buyer_price: u64,
        token_size: u64,
    ) -> Result<()> {
        cancel::auctioneer_cancel(ctx, buyer_price, token_size)
    }

    pub fn auctioneer_deposit<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerDeposit<'info>>,
        escrow_payment_bump: u8,
        amount: u64,
    ) -> Result<()> {
        deposit::auctioneer_deposit(ctx, escrow_payment_bump, amount)
    }

    pub fn auctioneer_execute_sale<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerExecuteSale<'info>>,
        escrow_payment_bump: u8,
        _free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        buyer_price: u64,
        token_size: u64,
    ) -> Result<()> {
        execute_sale::auctioneer_execute_sale(
            ctx,
            escrow_payment_bump,
            _free_trade_state_bump,
            program_as_signer_bump,
            buyer_price,
            token_size,
        )
    }

    pub fn auctioneer_sell<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerSell<'info>>,
        trade_state_bump: u8,
        free_trade_state_bump: u8,
        program_as_signer_bump: u8,
        token_size: u64,
    ) -> Result<()> {
        sell::auctioneer_sell(
            ctx,
            trade_state_bump,
            free_trade_state_bump,
            program_as_signer_bump,
            token_size,
        )
    }

    pub fn auctioneer_withdraw<'info>(
        ctx: Context<'_, '_, '_, 'info, AuctioneerWithdraw<'info>>,
        escrow_payment_bump: u8,
        amount: u64,
    ) -> Result<()> {
        withdraw::auctioneer_withdraw(ctx, escrow_payment_bump, amount)
    }

    pub fn close_escrow_account<'info>(
        ctx: Context<'_, '_, '_, 'info, CloseEscrowAccount<'info>>,
        escrow_payment_bump: u8,
    ) -> Result<()> {
        let auction_house_key = ctx.accounts.auction_house.key();
        let wallet_key = ctx.accounts.wallet.key();

        let escrow_signer_seeds = [
            PREFIX.as_bytes(),
            auction_house_key.as_ref(),
            wallet_key.as_ref(),
            &[escrow_payment_bump],
        ];

        invoke_signed(
            &system_instruction::transfer(
                &ctx.accounts.escrow_payment_account.key(),
                &ctx.accounts.wallet.key(),
                ctx.accounts.escrow_payment_account.lamports(),
            ),
            &[
                ctx.accounts.escrow_payment_account.to_account_info(),
                ctx.accounts.wallet.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&escrow_signer_seeds],
        )?;
        Ok(())
    }

    pub fn delegate_auctioneer<'info>(
        ctx: Context<'_, '_, '_, 'info, DelegateAuctioneer<'info>>,
    ) -> Result<()> {
        auctioneer::delegate_auctioneer(ctx)
    }
}

#[derive(Accounts)]
#[instruction(bump: u8, fee_payer_bump: u8, treasury_bump: u8)]
pub struct CreateAuctionHouse<'info> {
    pub treasury_mint: Account<'info, Mint>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub fee_withdrawal_destination: UncheckedAccount<'info>,
    #[account(mut)]
    pub treasury_withdrawal_destination: UncheckedAccount<'info>,
    pub treasury_withdrawal_destination_owner: UncheckedAccount<'info>,
    #[account(init, seeds=[PREFIX.as_bytes(), authority.key().as_ref(), treasury_mint.key().as_ref()], bump, space=AUCTION_HOUSE_SIZE, payer=payer)]
    pub auction_house: Account<'info, AuctionHouse>,
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), TREASURY.as_bytes()], bump)]
    pub auction_house_treasury: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateAuctionHouse<'info> {
    pub treasury_mint: Account<'info, Mint>,
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    pub new_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub fee_withdrawal_destination: UncheckedAccount<'info>,
    #[account(mut)]
    pub treasury_withdrawal_destination: UncheckedAccount<'info>,
    pub treasury_withdrawal_destination_owner: UncheckedAccount<'info>,
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), treasury_mint.key().as_ref()], bump=auction_house.bump, has_one=authority, has_one=treasury_mint)]
    pub auction_house: Account<'info, AuctionHouse>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct WithdrawFromTreasury<'info> {
    pub treasury_mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub treasury_withdrawal_destination: UncheckedAccount<'info>,
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), TREASURY.as_bytes()], bump=auction_house.treasury_bump)]
    pub auction_house_treasury: UncheckedAccount<'info>,
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), treasury_mint.key().as_ref()], bump=auction_house.bump, has_one=authority, has_one=treasury_mint, has_one=treasury_withdrawal_destination, has_one=auction_house_treasury)]
    pub auction_house: Account<'info, AuctionHouse>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFromFee<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub fee_withdrawal_destination: UncheckedAccount<'info>,
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    pub auction_house_fee_account: UncheckedAccount<'info>,
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.key().as_ref()], bump=auction_house.bump, has_one=authority, has_one=fee_withdrawal_destination, has_one=auction_house_fee_account)]
    pub auction_house: Account<'info, AuctionHouse>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(escrow_payment_bump: u8)]
pub struct CloseEscrowAccount<'info> {
    #[account(mut)]
    pub wallet: Signer<'info>,
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ],
        bump = escrow_payment_bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        bump = auction_house.bump
    )]
    pub auction_house: Account<'info, AuctionHouse>,
    pub system_program: Program<'info, System>,
}
