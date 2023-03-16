use cosmwasm_std::{Timestamp, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Basic configuration for the contract
/// The contract will have no admin so this will need to be set correctly
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub enable_cw20_receive: bool, // should the cw20 receive functionality be enabled? (cannot be changed later)
    pub set_withdraw_as_immutable: bool, // should the withdraw_address be updateable? (cannot be changed later)
    pub override_address: String,        // the deadman switch address and admin
    pub withdraw_address: String,        // the address whose funds are locked in this contract
    pub withdraw_delay_in_days: u64,     // withdraw delay in days
    pub native_denom: String,            // native chain denom - presumably ujuno
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Can be run by the withdrawal address
    /// Starts the withdraw process and creates a timestamp
    /// of when the funds will be ready for claim
    /// the denom_or_address field should match either the
    /// CW20 contract corresponding to the token to be withdrawn
    /// or the native denom to be withdrawn
    /// this can only be executed by the withdrawal_address
    StartWithdraw {
        denom_or_address: String,
        amount: Uint128,
    },
    /// When the NATIVE funds are ready to be claimed,
    /// this allows them to actually be claimed
    /// specify the native denom and amount
    /// this can only be executed by the withdrawal_address
    /// this also resets the timer once complete
    ExecuteNativeWithdraw { denom: String, amount: Uint128 },
    /// When the CW20 funds are ready to be claimed,
    /// this allows them to be claimed
    /// takes the address of the CW20 balance to be claimed
    /// this can only be executed by the withdrawal_address
    ExecuteCW20Withdraw { address: String, amount: Uint128 },
    /// When any CW20 funds custodied by
    /// this contract are claimable,
    /// this allows them to be claimed
    /// takes the address of the CW20 balance to be claimed
    /// this can only be executed by the withdrawal_address
    ExecuteEscrowCW20Withdraw { address: String, amount: Uint128 },
    /// If a withdrawal is in progress, cancel it
    /// this can only be executed by the override_address
    OverrideWithdraw {},
    /// Update the override_address
    /// this can only be executed by the override_address
    UpdateOverrideAddress { address: String },
    /// Update the withdrawal address
    /// this can only be executed by the override_address
    /// additionally, it can be turned off on instantiate
    UpdateWithdrawalAddress { address: String },
    ///
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// This returns the configured contract info
    GetConfig {},
    /// If a withdrawal has been initiated, this gets
    /// the timestamp that it will be ready to claim
    GetWithdrawalReadyTime {},
    /// Checks if a withdrawal is possible yet
    /// returns a bool response
    IsWithdrawalReady {},
    /// Checks if a withdrawal has been requested
    /// i.e. if the withdrawal requested is None
    GetWithdrawalRequested {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct WithdrawalTimestampResponse {
    pub withdrawal_ready_timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct WithdrawalReadyResponse {
    pub is_withdrawal_ready: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct WithdrawalRequestedResponse {
    pub withdrawal_requested: bool,
}
