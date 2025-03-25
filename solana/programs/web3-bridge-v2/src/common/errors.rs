use anchor_lang::prelude::*;

#[error_code]
pub enum XBridgeErrorCode {
    #[msg("UnSafe Covert")]
    UnSafeCovert,
    #[msg("User enter amount must equal user consumed")]
    AmountMustEqualConsumed,
    #[msg("Invalid swapType")]
    InvalidSwapType,
    #[msg("Invalid adaptor id")]
    InvalidAdaptorId,
    #[msg("Invalid to chain id")]
    InvalidToChainId,
    #[msg("transfer native-sol to user ATA Failed")]
    TransferNativeSolToUserATAFailed,
    #[msg("approve transfer authority delegate ATA Failed")]
    ApproveTransferAuthorityFailed,
    #[msg("invalid postman account")]
    InvalidPostmanAccount,
    #[msg("invalid Invalid mpc")]
    InvalidMPC,
    #[msg("not oracle proxy")]
    NotOracleProxy,
    #[msg("claim no oracle info")]
    OracleNoInfo,
    #[msg("claim to address err")]
    OracleToAddressErr,
    #[msg("claim token address err")]
    OracleTokenAddressErr,
    #[msg("has paid")]
    HASPAID,
    #[msg("Invalid accounts length")]
    InvalidAccountsLength,
    #[msg("Invalid account")]
    InvalidAccount,
    #[msg("Not Initialized")]
    NotInitialized,
    #[msg("Dex Data Mis Match")]
    DexDataMisMatch,
    #[msg("Invalid Dex Swap args: to address")]
    InvalidDexSwapArgsToAddress,
    #[msg("Invalid Dex Swap args: refund address")]
    InvalidDexSwapArgsRefundAddress,
    #[msg("Invalid Dex Swap args: from token address")]
    InvalidDexSwapArgsFromTokenAddress,
    #[msg("Invalid Dex Swap args: to token address")]
    InvalidDexSwapArgsToTokenAddress,
    #[msg("Invalid Dex Swap args: from amount")]
    InvalidDexSwapArgsFromAmount,
    #[msg("Invalid Dex Swap args: min return")]
    InvalidDexSwapArgsMinReturn,
    #[msg("Toswap Already Used")]
    ToswapAlreadyUsed,
    #[msg("Deserialization Error")]
    DeserializationError,
    #[msg("Data Too Long")]
    DataTooLong,
    #[msg("Srctxhash has used")]
    SrctxhashHasUsed,
    #[msg("Invalid admin")]
    InvalidAdmin,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Dex Min Return Not Reached")]
    DexMinReturnNotReached,
    #[msg("Refund failed")]
    RefundFailed,
    #[msg("Already Paused")]
    AlreadyPaused,
    #[msg("Not Paused")]
    NotPaused,
    #[msg("Invalid Pending Owner")]
    InvalidPendingOwner,
    #[msg("Already Initialized")]
    AlreadyInitialized,

    #[msg("invalid user address")]
    InvalidUserAddress,
    #[msg("invalid eth signature")]
    InvalidSignature,

    #[msg("account owned by wrong program")]
    AccountOwnedByWrongProgram,

    #[msg("Invalid commission rate")]
    InvalidCommissionRate,

    #[msg("Invalid commission token account")]
    InvalidCommissionTokenAccount,

    #[msg("Calculation error")]
    CalculationError,

    #[msg("Wsol Pda Invalid Mint Address")]
    WsolPdaInvalidMintAddress,

    #[msg("Wsol Pda Invalid Owner Address")]
    WsolPdaInvalidOwnerAddress,

    #[msg("Wsol Pda Invalid Account Amount")]
    WsolPdaInvalidAccountAmount,

    #[msg("Wsol Pda Failed To Decode TokenAccount")]
    WsolPdaFailedToDecodeTokenAccount,
}

#[error_code]
pub enum WormholeErrorCode {
    #[msg("wormhole call failed")]
    WormholeCallFailed,
    #[msg("InvalidRecipient")]
    /// Specified recipient has a bad chain ID or zero address.
    InvalidRecipient,
    #[msg("BumpNotFound")]
    BumpNotFound,
    #[msg("wormhole call transfer paylod failed")]
    WormholeCallTransferPayloadFailed,
    #[msg("wormhole call transfer failed")]
    WormholeCallTransferFailed,
}

#[error_code]
pub enum MesonErrorCode {
    #[msg("meson do not support swap type")]
    MesonDoNotSupportSwapType,
    #[msg("amount not match")]
    AmountNotMatch,
    #[msg("meson call failed")]
    MesonCallFailed,
    #[msg("InvalidRecipient")]
    /// Specified recipient has a bad chain ID or zero address.
    InvalidRecipient,
    #[msg("BumpNotFound")]
    BumpNotFound,
}

#[error_code]
pub enum DebridgedlnErrorCode {
    #[msg("debridge do not support swap type")]
    DebridgeDoNotSupportSwapType,
    #[msg("debridge call failed")]
    DebridgeCallFailed,
    #[msg("debridge pda mismatch")]
    PDAMismatch,
}


#[error_code]
pub enum CctpErrorCode {
    #[msg("cctp call failed")]
    CctpCallFailed,
    #[msg("cctp pda mismatch")]
    PDAMismatch,
    #[msg("Data length is insufficient")]
    DataError,
}

#[error_code]
pub enum AllBridgeErrorCode {
    #[msg("allbridge call failed")]
    AllbridgeCallFailed,
    #[msg("allbridge pda mismatch")]
    PDAMismatch,
    #[msg("Data length is insufficient")]
    DataError,
}

#[error_code]
pub enum BridgersErrorCode {
    #[msg("bridgers - invalid selectorId")]
    BridgersInvalidSelectorId,
}