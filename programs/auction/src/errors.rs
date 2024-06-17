use anchor_lang::prelude::*;

#[error_code]
pub enum AuctionHouseError {
    // 6000
    #[msg("PublicKeyMismatch")]
    PublicKeyMismatch,

    // 6001
    #[msg("UninitializedAccount")]
    UninitializedAccount,

    // 6002
    #[msg("IncorrectOwner")]
    IncorrectOwner,

    // 6003
    #[msg("NumericalOverflow")]
    NumericalOverflow,

    // 6004
    #[msg("No payer present on this txn")]
    NoPayerPresent,

    // 6005
    #[msg("Derived key invalid")]
    DerivedKeyInvalid,

    // 6006
    #[msg("Metadata doesn't exist")]
    MetadataDoesntExist,

    // 6007
    #[msg("Invalid token amount")]
    InvalidTokenAmount,

    // 6008
    #[msg("Both parties need to agree to this sale")]
    BothPartiesNeedToAgreeToSale,

    // 6009
    #[msg("Cannot match free sales unless the auction house or seller signs off")]
    CannotMatchFreeSalesWithoutAuctionHouseOrSellerSignoff,

    // 6010
    #[msg("This sale requires a signer")]
    SaleRequiresSigner,

    // 6011
    #[msg("Seller ata cannot have a delegate set")]
    SellerATACannotHaveDelegate,

    // 6012
    #[msg("Buyer ata cannot have a delegate set")]
    BuyerATACannotHaveDelegate,

    // 6013
    #[msg("No valid signer present")]
    NoValidSignerPresent,

    // 6014
    #[msg("BP must be less than or equal to 10000")]
    InvalidBasisPoints,

    // 6015
    #[msg("No Auctioneer program set.")]
    NoAuctioneerProgramSet,

    // 6016
    #[msg("Auction House not delegated.")]
    AuctionHouseNotDelegated,

    // 6017
    #[msg("Bump seed not in hash map.")]
    BumpSeedNotInHashMap,

    // 6018
    #[msg("The buyer trade state was unable to be initialized.")]
    BuyerTradeStateNotValid,

    // 6019
    #[msg("Amount of tokens available for purchase is less than the partial order amount.")]
    NotEnoughTokensAvailableForPurchase,

    // 6020
    #[msg("Auction House already delegated.")]
    AuctionHouseAlreadyDelegated,

    // 6021
    #[msg("Insufficient funds in escrow account.")]
    InsufficientFunds,
}
