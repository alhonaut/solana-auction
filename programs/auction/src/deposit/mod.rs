use anchor_lang::{prelude::*, solana_program::program::invoke, AnchorDeserialize};

use crate::{constants::*, errors::*, utils::*, AuctionHouse, *};

#[derive(Accounts, Clone)]
#[instruction(escrow_payment_bump: u8)]
pub struct AuctioneerDeposit<'info> {
    pub wallet: Signer<'info>,
    #[account(mut)]
    pub payment_account: UncheckedAccount<'info>,
    pub transfer_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            wallet.key().as_ref()
        ],
        bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,
    pub treasury_mint: Box<Account<'info, Mint>>,
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
        has_one=treasury_mint,
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
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            auctioneer_authority.key().as_ref()
        ],
        bump = auctioneer.bump
    )]
    pub auctioneer: Account<'info, Auctioneer>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn auctioneer_deposit<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerDeposit<'info>>,
    escrow_payment_bump: u8,
    amount: u64,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;

    if !auction_house.has_auctioneer {
        return Err(AuctionHouseError::NoAuctioneerProgramSet.into());
    }

    if escrow_payment_bump
        != *ctx
            .bumps
            .get("escrow_payment_account")
            .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?
    {
        return Err(AuctionHouseError::BumpSeedNotInHashMap.into());
    }

    let mut accounts: AuctioneerDeposit<'info> = (*ctx.accounts).clone();

    deposit_logic(&mut accounts, escrow_payment_bump, amount)
}

#[allow(clippy::needless_lifetimes)]
fn deposit_logic<'info>(
    accounts: &mut AuctioneerDeposit<'info>,
    escrow_payment_bump: u8,
    amount: u64,
) -> Result<()> {
    let wallet = &accounts.wallet;
    let payment_account = &accounts.payment_account;
    let transfer_authority = &accounts.transfer_authority;
    let escrow_payment_account = &accounts.escrow_payment_account;
    let authority = &accounts.authority;
    let auction_house = &accounts.auction_house;
    let auction_house_fee_account = &accounts.auction_house_fee_account;
    let treasury_mint = &accounts.treasury_mint;
    let system_program = &accounts.system_program;
    let token_program = &accounts.token_program;
    let rent = &accounts.rent;

    let auction_house_key = auction_house.key();
    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];
    let wallet_key = wallet.key();

    let escrow_signer_seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        wallet_key.as_ref(),
        &[escrow_payment_bump],
    ];

    let (fee_payer, fee_seeds) = get_fee_payer(
        authority,
        wallet.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;

    let is_native = treasury_mint.key() == spl_token::native_mint::id();

    create_program_token_account_if_not_present(
        escrow_payment_account,
        system_program,
        &fee_payer,
        token_program,
        treasury_mint,
        &auction_house.to_account_info(),
        rent,
        &escrow_signer_seeds,
        fee_seeds,
        is_native,
    )?;

    if !is_native {
        assert_is_ata(payment_account, &wallet.key(), &treasury_mint.key())?;
        invoke(
            &spl_token::instruction::transfer(
                token_program.key,
                &payment_account.key(),
                &escrow_payment_account.key(),
                &transfer_authority.key(),
                &[],
                amount,
            )?,
            &[
                escrow_payment_account.to_account_info(),
                payment_account.to_account_info(),
                token_program.to_account_info(),
                transfer_authority.to_account_info(),
            ],
        )?;
    } else {
        assert_keys_equal(payment_account.key(), wallet.key())?;

        let rent_shortfall = verify_deposit(escrow_payment_account.to_account_info(), 0)?;
        let checked_amount = amount
            .checked_add(rent_shortfall)
            .ok_or(AuctionHouseError::NumericalOverflow)?;

        invoke(
            &system_instruction::transfer(
                &payment_account.key(),
                &escrow_payment_account.key(),
                checked_amount,
            ),
            &[
                escrow_payment_account.to_account_info(),
                payment_account.to_account_info(),
                system_program.to_account_info(),
            ],
        )?;
    }

    Ok(())
}
