use crate::MintToken;
use anchor_lang::{prelude::*, solana_program::program::invoke};
use anchor_spl::{associated_token, token};
use mpl_token_metadata::instruction as mpl_instruction;

const ONE_TOKEN: u64 = 1;

pub fn create_associated_token_account(ctx: &Context<MintToken>) -> Result<()> {
    msg!("Creating Associated Token Account...");

    let cpi_program = ctx.accounts.associated_token_program.to_account_info();
    let cpi_accounts = associated_token::Create {
        payer: ctx.accounts.payer.to_account_info(),
        associated_token: ctx.accounts.associated_token_account.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
        mint: ctx.accounts.mint_account.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    associated_token::create(cpi_ctx)?;

    msg!(
        "Associated Token Account {} created successfully",
        ctx.accounts.associated_token_account.key()
    );

    Ok(())
}

pub fn mint_token_to_associated_token_account(ctx: &Context<MintToken>) -> Result<()> {
    msg!("Minting token to Associated Token Account...");

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_accounts = token::MintTo {
        mint: ctx.accounts.mint_account.to_account_info(),
        to: ctx.accounts.associated_token_account.to_account_info(),
        authority: ctx.accounts.mint_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::mint_to(cpi_ctx, ONE_TOKEN)?;

    msg!(
        "Successfully minted {} token to Associated Token Account {}",
        ONE_TOKEN,
        ctx.accounts.associated_token_account.key()
    );

    Ok(())
}

pub fn create_master_edition_account(
    ctx: &Context<MintToken>,
    max_supply: Option<u64>,
) -> Result<()> {
    msg!("Creating Master Edition Account...");

    invoke(
        &mpl_instruction::create_master_edition_v3(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.edition_account.key(),
            ctx.accounts.mint_account.key(),
            ctx.accounts.update_authority.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.metadata_account.key(),
            ctx.accounts.payer.key(),
            max_supply,
        ),
        &[
            ctx.accounts.edition_account.to_account_info(),
            ctx.accounts.mint_account.to_account_info(),
            ctx.accounts.update_authority.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
    )?;

    msg!(
        "Master Edition Account {} created successfully",
        ctx.accounts.edition_account.key()
    );

    Ok(())
}
