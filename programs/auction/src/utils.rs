use crate::{errors::AuctionHouseError, AuctionHouse, PREFIX};

use anchor_lang::{
    prelude::*,
    solana_program::{
        program::invoke_signed,
        program_memory::{sol_memcmp, sol_memset},
        program_pack::{IsInitialized, Pack},
        pubkey::PUBKEY_BYTES,
        system_instruction,
    },
};
use anchor_spl::token::{Mint, Token, TokenAccount};
use arrayref::array_ref;
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount};
use spl_token::{instruction::initialize_account2, state::Account as SplAccount};
use std::{convert::TryInto, slice::Iter};

pub fn assert_is_ata(ata: &AccountInfo, wallet: &Pubkey, mint: &Pubkey) -> Result<SplAccount> {
    assert_owned_by(ata, &spl_token::id())?;
    let ata_account: SplAccount = assert_initialized(ata)?;
    assert_keys_equal(ata_account.owner, *wallet)?;
    assert_keys_equal(ata_account.mint, *mint)?;

    Ok(ata_account)
}

pub fn make_ata<'a>(
    ata: AccountInfo<'a>,
    wallet: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    fee_payer: AccountInfo<'a>,
    ata_program: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    rent: AccountInfo<'a>,
    fee_payer_seeds: &[&[u8]],
) -> Result<()> {
    let as_arr = [fee_payer_seeds];

    let seeds: &[&[&[u8]]] = if !fee_payer_seeds.is_empty() {
        &as_arr
    } else {
        &[]
    };

    invoke_signed(
        &spl_associated_token_account::instruction::create_associated_token_account(
            fee_payer.key,
            wallet.key,
            mint.key,
            &spl_token::ID,
        ),
        &[
            ata,
            wallet,
            mint,
            fee_payer,
            ata_program,
            system_program,
            rent,
            token_program,
        ],
        seeds,
    )?;

    Ok(())
}

pub fn assert_metadata_valid<'a>(
    metadata: &UncheckedAccount,
    token_account: &anchor_lang::prelude::Account<'a, TokenAccount>,
) -> Result<()> {
    assert_derivation(
        &mpl_token_metadata::id(),
        &metadata.to_account_info(),
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            token_account.mint.as_ref(),
        ],
    )?;

    if metadata.data_is_empty() {
        return Err(AuctionHouseError::MetadataDoesntExist.into());
    }
    Ok(())
}

pub fn get_fee_payer<'a, 'b>(
    authority: &UncheckedAccount,
    wallet: AccountInfo<'a>,
    auction_house_fee_account: AccountInfo<'a>,
    auction_house_seeds: &'b [&'b [u8]],
) -> Result<(AccountInfo<'a>, &'b [&'b [u8]])> {
    let mut seeds: &[&[u8]] = &[];
    let fee_payer: AccountInfo;
    if authority.to_account_info().is_signer {
        seeds = auction_house_seeds;
        fee_payer = auction_house_fee_account;
    } else if wallet.is_signer {
        fee_payer = wallet
    } else {
        return Err(AuctionHouseError::NoPayerPresent.into());
    };

    Ok((fee_payer, seeds))
}

pub fn assert_keys_equal(key1: Pubkey, key2: Pubkey) -> Result<()> {
    if sol_memcmp(key1.as_ref(), key2.as_ref(), PUBKEY_BYTES) != 0 {
        err!(AuctionHouseError::PublicKeyMismatch)
    } else {
        Ok(())
    }
}

pub fn assert_initialized<T: Pack + IsInitialized>(account_info: &AccountInfo) -> Result<T> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        err!(AuctionHouseError::UninitializedAccount)
    } else {
        Ok(account)
    }
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<()> {
    if account.owner != owner {
        err!(AuctionHouseError::IncorrectOwner)
    } else {
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn pay_auction_house_fees<'a>(
    auction_house: &anchor_lang::prelude::Account<'a, AuctionHouse>,
    auction_house_treasury: &AccountInfo<'a>,
    escrow_payment_account: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    signer_seeds: &[&[u8]],
    size: u64,
    is_native: bool,
) -> Result<u64> {
    let fees = auction_house.seller_fee_basis_points;
    let total_fee = (fees as u128)
        .checked_mul(size as u128)
        .ok_or(AuctionHouseError::NumericalOverflow)?
        .checked_div(10000)
        .ok_or(AuctionHouseError::NumericalOverflow)? as u64;
    if !is_native {
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                escrow_payment_account.key,
                auction_house_treasury.key,
                &auction_house.key(),
                &[],
                total_fee,
            )?,
            &[
                escrow_payment_account.clone(),
                auction_house_treasury.clone(),
                token_program.clone(),
                auction_house.to_account_info(),
            ],
            &[signer_seeds],
        )?;
    } else {
        invoke_signed(
            &system_instruction::transfer(
                escrow_payment_account.key,
                auction_house_treasury.key,
                total_fee,
            ),
            &[
                escrow_payment_account.clone(),
                auction_house_treasury.clone(),
                system_program.clone(),
            ],
            &[signer_seeds],
        )?;
    }
    Ok(total_fee)
}

pub fn create_program_token_account_if_not_present<'a>(
    payment_account: &UncheckedAccount<'a>,
    system_program: &Program<'a, System>,
    fee_payer: &AccountInfo<'a>,
    token_program: &Program<'a, Token>,
    treasury_mint: &anchor_lang::prelude::Account<'a, Mint>,
    owner: &AccountInfo<'a>,
    rent: &Sysvar<'a, Rent>,
    signer_seeds: &[&[u8]],
    fee_seeds: &[&[u8]],
    is_native: bool,
) -> Result<()> {
    if !is_native && payment_account.data_is_empty() {
        create_or_allocate_account_raw(
            *token_program.key,
            &payment_account.to_account_info(),
            &rent.to_account_info(),
            system_program,
            fee_payer,
            spl_token::state::Account::LEN,
            fee_seeds,
            signer_seeds,
        )?;
        invoke_signed(
            &initialize_account2(
                token_program.key,
                &payment_account.key(),
                &treasury_mint.key(),
                &owner.key(),
            )
            .unwrap(),
            &[
                token_program.to_account_info(),
                treasury_mint.to_account_info(),
                payment_account.to_account_info(),
                rent.to_account_info(),
                owner.clone(),
            ],
            &[signer_seeds],
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn pay_creator_fees<'a>(
    remaining_accounts: &mut Iter<AccountInfo<'a>>,
    metadata_info: &AccountInfo<'a>,
    escrow_payment_account: &AccountInfo<'a>,
    payment_account_owner: &AccountInfo<'a>,
    fee_payer: &AccountInfo<'a>,
    treasury_mint: &AccountInfo<'a>,
    ata_program: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    signer_seeds: &[&[u8]],
    fee_payer_seeds: &[&[u8]],
    size: u64,
    is_native: bool,
) -> Result<u64> {
    let metadata = Metadata::from_account_info(metadata_info)?;
    let fees = metadata.data.seller_fee_basis_points;
    let total_fee = (fees as u128)
        .checked_mul(size as u128)
        .ok_or(AuctionHouseError::NumericalOverflow)?
        .checked_div(10000)
        .ok_or(AuctionHouseError::NumericalOverflow)? as u64;
    let mut remaining_fee = total_fee;
    let remaining_size = size
        .checked_sub(total_fee)
        .ok_or(AuctionHouseError::NumericalOverflow)?;
    match metadata.data.creators {
        Some(creators) => {
            for creator in creators {
                let pct = creator.share as u128;
                let creator_fee =
                    pct.checked_mul(total_fee as u128)
                        .ok_or(AuctionHouseError::NumericalOverflow)?
                        .checked_div(100)
                        .ok_or(AuctionHouseError::NumericalOverflow)? as u64;
                let current_creator_info = next_account_info(remaining_accounts)?;
                let creator_rent_minimum =
                    Rent::get()?.minimum_balance(current_creator_info.data.borrow().len());
                if is_native
                    && ((creator_fee + **current_creator_info.lamports.borrow())
                        < creator_rent_minimum)
                {
                    msg!(
                        "cannot pay creator {} {} lamports since balance violates rent exempt minimum",
                        current_creator_info.key,
                        creator_fee
                    );
                    continue;
                }

                remaining_fee = remaining_fee
                    .checked_sub(creator_fee)
                    .ok_or(AuctionHouseError::NumericalOverflow)?;
                assert_keys_equal(creator.address, *current_creator_info.key)?;
                if !is_native {
                    let current_creator_token_account_info = next_account_info(remaining_accounts)?;
                    if current_creator_token_account_info.data_is_empty() {
                        make_ata(
                            current_creator_token_account_info.to_account_info(),
                            current_creator_info.to_account_info(),
                            treasury_mint.to_account_info(),
                            fee_payer.to_account_info(),
                            ata_program.to_account_info(),
                            token_program.to_account_info(),
                            system_program.to_account_info(),
                            rent.to_account_info(),
                            fee_payer_seeds,
                        )?;
                    }
                    assert_is_ata(
                        current_creator_token_account_info,
                        current_creator_info.key,
                        &treasury_mint.key(),
                    )?;
                    if creator_fee > 0 {
                        invoke_signed(
                            &spl_token::instruction::transfer(
                                token_program.key,
                                escrow_payment_account.key,
                                current_creator_token_account_info.key,
                                payment_account_owner.key,
                                &[],
                                creator_fee,
                            )?,
                            &[
                                escrow_payment_account.clone(),
                                current_creator_token_account_info.clone(),
                                token_program.clone(),
                                payment_account_owner.clone(),
                            ],
                            &[signer_seeds],
                        )?;
                    }
                } else if creator_fee > 0 {
                    invoke_signed(
                        &system_instruction::transfer(
                            escrow_payment_account.key,
                            current_creator_info.key,
                            creator_fee,
                        ),
                        &[
                            escrow_payment_account.clone(),
                            current_creator_info.clone(),
                            system_program.clone(),
                        ],
                        &[signer_seeds],
                    )?;
                }
            }
        }
        None => {
            msg!("No creators found in metadata");
        }
    }

    Ok(remaining_size
        .checked_add(remaining_fee)
        .ok_or(AuctionHouseError::NumericalOverflow)?)
}

pub fn get_mint_from_token_account(token_account_info: &AccountInfo) -> Result<Pubkey> {
    let data = token_account_info.try_borrow_data()?;
    let mint_data = array_ref![data, 0, 32];
    Ok(Pubkey::new_from_array(*mint_data))
}

pub fn get_delegate_from_token_account(token_account_info: &AccountInfo) -> Result<Option<Pubkey>> {
    let data = token_account_info.try_borrow_data()?;
    let key_data = array_ref![data, 76, 32];
    let coption_data = u32::from_le_bytes(*array_ref![data, 72, 4]);
    if coption_data == 0 {
        Ok(None)
    } else {
        Ok(Some(Pubkey::new_from_array(*key_data)))
    }
}

#[inline(always)]
pub fn create_or_allocate_account_raw<'a>(
    program_id: Pubkey,
    new_account_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    size: usize,
    signer_seeds: &[&[u8]],
    new_acct_seeds: &[&[u8]],
) -> Result<()> {
    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(size)
        .max(1)
        .saturating_sub(new_account_info.lamports());

    if required_lamports > 0 {
        msg!("Transfer {} lamports to the new account", required_lamports);

        let as_arr = [signer_seeds];
        let seeds: &[&[&[u8]]] = if !signer_seeds.is_empty() {
            &as_arr
        } else {
            &[]
        };

        invoke_signed(
            &system_instruction::transfer(payer_info.key, new_account_info.key, required_lamports),
            &[
                payer_info.clone(),
                new_account_info.clone(),
                system_program_info.clone(),
            ],
            seeds,
        )?;
    }

    let accounts = &[new_account_info.clone(), system_program_info.clone()];

    msg!("Allocate space for the account {}", new_account_info.key);
    invoke_signed(
        &system_instruction::allocate(new_account_info.key, size.try_into().unwrap()),
        accounts,
        &[new_acct_seeds],
    )?;

    msg!("Assign the account to the owning program");
    invoke_signed(
        &system_instruction::assign(new_account_info.key, &program_id),
        accounts,
        &[new_acct_seeds],
    )?;
    msg!("Completed assignation!");

    Ok(())
}

/// Receives a program id, account info, and seeds and verifies that the pubkey of the account
/// is the PDA generated by the seeds and the program id.
/// Returns the bump seed.
pub fn assert_derivation(program_id: &Pubkey, account: &AccountInfo, path: &[&[u8]]) -> Result<u8> {
    let (key, bump) = Pubkey::find_program_address(path, program_id);
    if key != *account.key {
        return Err(AuctionHouseError::DerivedKeyInvalid.into());
    }
    Ok(bump)
}

pub fn assert_valid_trade_state(
    wallet: &Pubkey,
    auction_house: &Account<AuctionHouse>,
    buyer_price: u64,
    token_size: u64,
    trade_state: &AccountInfo,
    mint: &Pubkey,
    token_holder: &Pubkey,
    ts_bump: u8,
) -> Result<u8> {
    let ah_pubkey = &auction_house.key();
    let mint_bytes = mint.as_ref();
    let treasury_mint_bytes = auction_house.treasury_mint.as_ref();
    let buyer_price_bytes = buyer_price.to_le_bytes();
    let token_size_bytes = token_size.to_le_bytes();
    let wallet_bytes = wallet.as_ref();
    let auction_house_key_bytes = ah_pubkey.as_ref();
    let pfix = PREFIX.as_bytes();
    let token_holder_bytes = token_holder.as_ref();
    let canonical_bump = assert_derivation(
        &crate::id(),
        trade_state,
        &[
            pfix,
            wallet_bytes,
            auction_house_key_bytes,
            token_holder_bytes,
            treasury_mint_bytes,
            mint_bytes,
            &buyer_price_bytes,
            &token_size_bytes,
        ],
    );

    let canonical_public_bump = assert_derivation(
        &crate::id(),
        trade_state,
        &[
            pfix,
            wallet_bytes,
            auction_house_key_bytes,
            treasury_mint_bytes,
            mint_bytes,
            &buyer_price_bytes,
            &token_size_bytes,
        ],
    );

    match (canonical_public_bump, canonical_bump) {
        (Ok(public), Err(_)) if public == ts_bump => Ok(public),
        (Err(_), Ok(bump)) if bump == ts_bump => Ok(bump),
        _ => Err(AuctionHouseError::DerivedKeyInvalid.into()),
    }
}

pub fn verify_withdrawal(account: AccountInfo, amount: u64) -> Result<u64> {
    let rent_minimum = (Rent::get()?).minimum_balance(account.data_len());
    let diff = account
        .lamports()
        .checked_sub(amount)
        .ok_or(AuctionHouseError::InsufficientFunds)?;

    Ok(rent_minimum.saturating_sub(diff))
}

pub fn verify_deposit(account: AccountInfo, amount: u64) -> Result<u64> {
    let rent_minimum = (Rent::get()?).minimum_balance(account.data_len());
    let total = account
        .lamports()
        .checked_add(amount)
        .ok_or(AuctionHouseError::NumericalOverflow)?;

    Ok(rent_minimum.saturating_sub(total))
}

pub fn close_account<'a>(
    source_account: &AccountInfo<'a>,
    receiver_account: &AccountInfo<'a>,
) -> Result<()> {
    let current_lamports = source_account.lamports();
    let account_data_size = source_account.data_len();

    **source_account.lamports.borrow_mut() = 0;
    **receiver_account.lamports.borrow_mut() = receiver_account
        .lamports()
        .checked_add(current_lamports)
        .ok_or(AuctionHouseError::NumericalOverflow)?;

    #[allow(clippy::explicit_auto_deref)]
    sol_memset(*source_account.try_borrow_mut_data()?, 0, account_data_size);

    Ok(())
}
