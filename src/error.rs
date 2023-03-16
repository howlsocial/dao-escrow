use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Error - CW20 Receive not enabled")]
    CW20ReceiveDisabled,

    #[error("Error while calculating CW20 balance")]
    CW20BalanceError,

    #[error("Withdrawal not yet requested")]
    WithdrawalNotRequested,

    #[error("Withdrawal not ready - wait until after timeout has passed")]
    WithdrawalNotReady {},

    #[error("Withdrawal denom does not match the requested one")]
    WithdrawalDenomMismatch {},

    #[error("Withdrawal CW20 contract address does not match the requested one")]
    WithdrawalCW20Mismatch {},

    #[error("Withdrawal address does not match the requested one")]
    WithdrawalAddressMismatch {},

    #[error("Withdrawal amount does not match the requested total")]
    WithdrawalAmountMismatch {},

    #[error("Contract balance is too small to execute")]
    InsufficientContractBalance {},

    #[error("A native balance was not found in the Contract balances")]
    NoNativeBalance {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("The Withdrawal address was set as immutable on contract instantiation")]
    WithdrawalAddressIsImmutable {},
}
