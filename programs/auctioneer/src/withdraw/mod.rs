use anchor_lang::{
    prelude::*, solana_program::program::invoke_signed, AnchorDeserialize, InstructionData,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token},
};

use auction_house::{
    self,
    constants::{AUCTIONEER, FEE_PAYER, PREFIX},
    cpi::accounts::AuctioneerWithdraw as AHWithdraw,
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse,
};

#[derive(Accounts, Clone)]
#[instruction(
    escrow_payment_bump: u8,
    auctioneer_authority_bump: u8
)]
pub struct AuctioneerWithdraw<'info> {
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    pub wallet: Signer<'info>,
    #[account(mut)]
    pub receipt_account: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = escrow_payment_bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,
    pub treasury_mint: Box<Account<'info, Mint>>,
    pub authority: UncheckedAccount<'info>,
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.bump,
        has_one = treasury_mint,
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
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref()
        ],
        bump = auctioneer_authority_bump
    )]
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
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn auctioneer_withdraw(
    ctx: Context<AuctioneerWithdraw>,
    escrow_payment_bump: u8,
    auctioneer_authority_bump: u8,
    amount: u64,
) -> Result<()> {
    let cpi_program = ctx.accounts.auction_house_program.to_account_info();
    let cpi_accounts = AHWithdraw {
        wallet: ctx.accounts.wallet.to_account_info(),
        receipt_account: ctx.accounts.receipt_account.to_account_info(),
        escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
        treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        auctioneer_authority: ctx.accounts.auctioneer_authority.to_account_info(),
        auctioneer: ctx.accounts.auctioneer.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let withdraw_data = auction_house::instruction::AuctioneerWithdraw {
        escrow_payment_bump,
        amount,
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
        data: withdraw_data.data(),
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

    Ok(())
}
