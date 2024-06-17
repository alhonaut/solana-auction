use crate::CreateToken;
use anchor_lang::system_program;
use anchor_lang::{prelude::*, solana_program::program::invoke};
use anchor_spl::token;
use mpl_token_metadata::instruction as mpl_instruction;
use mpl_token_metadata::state::Creator;

const DECIMALS: u8 = 0;

pub fn create_mint_account(ctx: &Context<CreateToken>) -> Result<()> {
    msg!("Creating Mint Account...");

    let space = token::Mint::LEN as u64;
    let lamports = Rent::get()?.minimum_balance(space as usize);
    let owner = &ctx.accounts.token_program.key();

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_accounts = system_program::CreateAccount {
        from: ctx.accounts.mint_authority.to_account_info(),
        to: ctx.accounts.mint_account.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    system_program::create_account(cpi_ctx, lamports, space, owner)?;

    msg!(
        "Mint Account {} created successfully",
        ctx.accounts.mint_account.key()
    );

    Ok(())
}

pub fn initialize_mint_account(ctx: &Context<CreateToken>) -> Result<()> {
    msg!("Initializing Mint Account...");

    let authority = &ctx.accounts.mint_authority.key();
    let freeze_authority = Some(authority);

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_accounts = token::InitializeMint {
        mint: ctx.accounts.mint_account.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::initialize_mint(cpi_ctx, DECIMALS, authority, freeze_authority)?;

    msg!(
        "Mint Account {} initialized successfully",
        ctx.accounts.mint_account.key()
    );

    Ok(())
}

pub fn create_metadata_account(
    ctx: &Context<CreateToken>,
    name: String,
    symbol: String,
    uri: String,
    creators: Option<Vec<Creator>>,
    seller_fee_basis_points: u16,
    is_mutable: bool,
) -> Result<()> {
    msg!("Creating Metadata Account...");

    invoke(
        &mpl_instruction::create_metadata_accounts_v3(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.metadata_account.key(),
            ctx.accounts.mint_account.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.payer.key(),
            ctx.accounts.update_authority.key(),
            name,
            symbol,
            uri,
            creators,
            seller_fee_basis_points,
            true,
            is_mutable,
            None,
            None,
            None,
        ),
        &[
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.mint_account.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.update_authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
    )?;

    msg!(
        "Metadata Account {} created successfully",
        ctx.accounts.metadata_account.key()
    );

    Ok(())
}
