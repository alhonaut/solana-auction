use crate::{constants::*, errors::*, utils::*, AuctionHouse, Auctioneer, *};
use anchor_lang::{prelude::*, solana_program::program_pack::Pack, AnchorDeserialize};
use spl_token::state::Account as SplAccount;

#[derive(Accounts)]
#[instruction(
    escrow_payment_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64
)]
pub struct AuctioneerExecuteSale<'info> {
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    pub token_mint: UncheckedAccount<'info>,
    pub metadata: UncheckedAccount<'info>,
    pub treasury_mint: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            buyer.key().as_ref()
        ],
        bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub seller_payment_receipt_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub buyer_receipt_token_account: UncheckedAccount<'info>,
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
        has_one=auction_house_treasury,
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
            auction_house.key().as_ref(),
            TREASURY.as_bytes()
        ],
        bump=auction_house.treasury_bump
    )]
    pub auction_house_treasury: UncheckedAccount<'info>,
    #[account(mut)]
    pub buyer_trade_state: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            seller.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_mint.key().as_ref(),
            &u64::MAX.to_le_bytes(),
            &token_size.to_le_bytes()
        ],
        bump=seller_trade_state.to_account_info().data.borrow()[0]
    )]
    pub seller_trade_state: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            seller.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_mint.key().as_ref(),
            &0u64.to_le_bytes(),
            &token_size.to_le_bytes()
        ],
        bump
    )]
    pub free_trade_state: UncheckedAccount<'info>,
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            auctioneer_authority.key().as_ref()
        ],
        bump = auctioneer.bump
    )]
    pub auctioneer: Box<Account<'info, Auctioneer>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(
        seeds=[
            PREFIX.as_bytes(), SIGNER.as_bytes()
        ],
        bump
    )]
    pub program_as_signer: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
}

pub fn auctioneer_execute_sale<'info>(
    ctx: Context<'_, '_, '_, 'info, AuctioneerExecuteSale<'info>>,
    escrow_payment_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let auction_house = &ctx.accounts.auction_house;

    if !auction_house.has_auctioneer {
        return Err(AuctionHouseError::NoAuctioneerProgramSet.into());
    }

    let escrow_canonical_bump = *ctx
        .bumps
        .get("escrow_payment_account")
        .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;
    let free_trade_state_canonical_bump = *ctx
        .bumps
        .get("free_trade_state")
        .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;
    let program_as_signer_canonical_bump = *ctx
        .bumps
        .get("program_as_signer")
        .ok_or(AuctionHouseError::BumpSeedNotInHashMap)?;

    if (escrow_canonical_bump != escrow_payment_bump)
        || (free_trade_state_canonical_bump != free_trade_state_bump)
        || (program_as_signer_canonical_bump != program_as_signer_bump)
    {
        return Err(AuctionHouseError::BumpSeedNotInHashMap.into());
    }

    auctioneer_execute_sale_logic(
        ctx.accounts,
        ctx.remaining_accounts,
        escrow_payment_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        buyer_price,
        token_size,
    )
}

#[inline(never)]
fn auctioneer_execute_sale_logic<'c, 'info>(
    accounts: &mut AuctioneerExecuteSale<'info>,
    remaining_accounts: &'c [AccountInfo<'info>],
    escrow_payment_bump: u8,
    _free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
) -> Result<()> {
    let buyer = &accounts.buyer;
    let seller = &accounts.seller;
    let token_account = &accounts.token_account;
    let token_mint = &accounts.token_mint;
    let metadata = &accounts.metadata;
    let treasury_mint = &accounts.treasury_mint;
    let seller_payment_receipt_account = &accounts.seller_payment_receipt_account;
    let buyer_receipt_token_account = &accounts.buyer_receipt_token_account;
    let escrow_payment_account = &accounts.escrow_payment_account;
    let authority = &accounts.authority;
    let auction_house = &accounts.auction_house;
    let auction_house_fee_account = &accounts.auction_house_fee_account;
    let auction_house_treasury = &accounts.auction_house_treasury;
    let buyer_trade_state = &accounts.buyer_trade_state;
    let seller_trade_state = &accounts.seller_trade_state;
    let free_trade_state = &accounts.free_trade_state;
    let token_program = &accounts.token_program;
    let system_program = &accounts.system_program;
    let associated_token_program = &accounts.associated_token_program;
    let program_as_signer = &accounts.program_as_signer;
    let rent = &accounts.rent;

    let metadata_clone = metadata.to_account_info();
    let escrow_clone = escrow_payment_account.to_account_info();
    let auction_house_clone = auction_house.to_account_info();
    let ata_clone = associated_token_program.to_account_info();
    let token_clone = token_program.to_account_info();
    let sys_clone = system_program.to_account_info();
    let rent_clone = rent.to_account_info();
    let treasury_clone = auction_house_treasury.to_account_info();
    let authority_clone = authority.to_account_info();
    let buyer_receipt_clone = buyer_receipt_token_account.to_account_info();
    let token_account_clone = token_account.to_account_info();

    let is_native = treasury_mint.key() == spl_token::native_mint::id();

    if buyer_price == 0 && !authority_clone.is_signer && !seller.is_signer {
        return Err(
            AuctionHouseError::CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff.into(),
        );
    }

    let token_account_mint = get_mint_from_token_account(&token_account_clone)?;

    assert_keys_equal(token_mint.key(), token_account_mint)?;
    let delegate = get_delegate_from_token_account(&token_account_clone)?;
    if let Some(d) = delegate {
        assert_keys_equal(program_as_signer.key(), d)?;
    } else {
        msg!("No delegate detected on token account.");
        return Err(AuctionHouseError::BothPartiesNeedToAgreeToSale.into());
    }

    let ts_bump = if buyer_trade_state.data_len() > 0 {
        buyer_trade_state.try_borrow_data()?[0]
    } else {
        return Err(AuctionHouseError::BuyerTradeStateNotValid.into());
    };

    if ts_bump == 0
        || seller_trade_state.data_len() == 0
        || seller_trade_state.try_borrow_data()?[0] == 0
    {
        return Err(AuctionHouseError::BothPartiesNeedToAgreeToSale.into());
    }

    let token_account_data = SplAccount::unpack(&token_account.data.borrow())?;

    assert_valid_trade_state(
        &buyer.key(),
        auction_house,
        buyer_price,
        token_size,
        buyer_trade_state,
        &token_mint.key(),
        &token_account.key(),
        ts_bump,
    )?;

    if token_account_data.amount < token_size {
        return Err(AuctionHouseError::NotEnoughTokensAvailableForPurchase.into());
    };

    let auction_house_key = auction_house.key();
    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];

    let wallet_to_use = if buyer.is_signer { buyer } else { seller };

    let (fee_payer, fee_payer_seeds) = get_fee_payer(
        authority,
        wallet_to_use.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;
    let fee_payer_clone = fee_payer.to_account_info();

    assert_is_ata(
        &token_account.to_account_info(),
        &seller.key(),
        &token_account_mint,
    )?;
    assert_derivation(
        &mpl_token_metadata::id(),
        &metadata.to_account_info(),
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            token_account_mint.as_ref(),
        ],
    )?;

    if is_native {
        let rent_shortfall =
            verify_withdrawal(escrow_payment_account.to_account_info(), buyer_price)?;
        if rent_shortfall > 0 {
            invoke_signed(
                &system_instruction::transfer(
                    fee_payer.key,
                    escrow_payment_account.key,
                    rent_shortfall,
                ),
                &[
                    fee_payer.to_account_info(),
                    escrow_payment_account.to_account_info(),
                    system_program.to_account_info(),
                ],
                &[fee_payer_seeds],
            )?;
        }
    }

    if metadata.data_is_empty() {
        return Err(AuctionHouseError::MetadataDoesntExist.into());
    }

    let auction_house_key = auction_house.key();
    let wallet_key = buyer.key();
    let escrow_signer_seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        wallet_key.as_ref(),
        &[escrow_payment_bump],
    ];

    let ah_seeds = [
        PREFIX.as_bytes(),
        auction_house.creator.as_ref(),
        auction_house.treasury_mint.as_ref(),
        &[auction_house.bump],
    ];

    let signer_seeds_for_royalties = if is_native {
        escrow_signer_seeds
    } else {
        ah_seeds
    };

    let remaining_accounts = &mut remaining_accounts.iter();

    let buyer_leftover_after_royalties = pay_creator_fees(
        remaining_accounts,
        &metadata_clone,
        &escrow_clone,
        &auction_house_clone,
        &fee_payer_clone,
        treasury_mint,
        &ata_clone,
        &token_clone,
        &sys_clone,
        &rent_clone,
        &signer_seeds_for_royalties,
        fee_payer_seeds,
        buyer_price,
        is_native,
    )?;

    let auction_house_fee_paid = pay_auction_house_fees(
        auction_house,
        &treasury_clone,
        &escrow_clone,
        &token_clone,
        &sys_clone,
        &signer_seeds_for_royalties,
        buyer_price,
        is_native,
    )?;

    let buyer_leftover_after_royalties_and_house_fee = buyer_leftover_after_royalties
        .checked_sub(auction_house_fee_paid)
        .ok_or(AuctionHouseError::NumericalOverflow)?;

    if !is_native {
        if seller_payment_receipt_account.data_is_empty() {
            make_ata(
                seller_payment_receipt_account.to_account_info(),
                seller.to_account_info(),
                treasury_mint.to_account_info(),
                fee_payer.to_account_info(),
                associated_token_program.to_account_info(),
                token_program.to_account_info(),
                system_program.to_account_info(),
                rent.to_account_info(),
                fee_payer_seeds,
            )?;
        }

        let seller_rec_acct = assert_is_ata(
            &seller_payment_receipt_account.to_account_info(),
            &seller.key(),
            &treasury_mint.key(),
        )?;

        if seller_rec_acct.delegate.is_some() {
            return Err(AuctionHouseError::SellerATACannotHaveDelegate.into());
        }

        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                &escrow_payment_account.key(),
                &seller_payment_receipt_account.key(),
                &auction_house.key(),
                &[],
                buyer_leftover_after_royalties_and_house_fee,
            )?,
            &[
                escrow_payment_account.to_account_info(),
                seller_payment_receipt_account.to_account_info(),
                token_program.to_account_info(),
                auction_house.to_account_info(),
            ],
            &[&ah_seeds],
        )?;
    } else {
        assert_keys_equal(seller_payment_receipt_account.key(), seller.key())?;
        invoke_signed(
            &system_instruction::transfer(
                escrow_payment_account.key,
                seller_payment_receipt_account.key,
                buyer_leftover_after_royalties_and_house_fee,
            ),
            &[
                escrow_payment_account.to_account_info(),
                seller_payment_receipt_account.to_account_info(),
                system_program.to_account_info(),
            ],
            &[&escrow_signer_seeds],
        )?;
    }

    if buyer_receipt_token_account.data_is_empty() {
        make_ata(
            buyer_receipt_token_account.to_account_info(),
            buyer.to_account_info(),
            token_mint.to_account_info(),
            fee_payer.to_account_info(),
            associated_token_program.to_account_info(),
            token_program.to_account_info(),
            system_program.to_account_info(),
            rent.to_account_info(),
            fee_payer_seeds,
        )?;
    } else {
        let data = buyer_receipt_token_account.try_borrow_data()?;
        let token_account = TokenAccount::try_deserialize(&mut data.as_ref())?;
        if &token_account.owner != buyer.key {
            return Err(AuctionHouseError::IncorrectOwner.into());
        }
    }

    let buyer_rec_acct = assert_is_ata(&buyer_receipt_clone, &buyer.key(), &token_mint.key())?;

    if buyer_rec_acct.delegate.is_some() {
        return Err(AuctionHouseError::BuyerATACannotHaveDelegate.into());
    }

    let program_as_signer_seeds = [
        PREFIX.as_bytes(),
        SIGNER.as_bytes(),
        &[program_as_signer_bump],
    ];

    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            &token_account.key(),
            &buyer_receipt_token_account.key(),
            &program_as_signer.key(),
            &[],
            token_size,
        )?,
        &[
            token_account.to_account_info(),
            buyer_receipt_clone,
            program_as_signer.to_account_info(),
            token_clone,
        ],
        &[&program_as_signer_seeds],
    )?;

    close_account(
        &buyer_trade_state.to_account_info(),
        &fee_payer.to_account_info(),
    )?;

    let token_account_data = SplAccount::unpack(&token_account.data.borrow())?;
    if token_account_data.delegated_amount == 0 {
        close_account(
            &seller_trade_state.to_account_info(),
            &fee_payer.to_account_info(),
        )?;

        if free_trade_state.lamports() > 0 {
            close_account(
                &free_trade_state.to_account_info(),
                &fee_payer.to_account_info(),
            )?;
        }
    }
    Ok(())
}
