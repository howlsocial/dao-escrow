use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub enable_cw20_receive: bool,
    pub set_withdraw_as_immutable: bool,
    pub set_override_as_immutable: bool,
    pub withdraw_address: Addr,
    pub override_address: Addr,
    pub withdraw_delay_in_days: u64,
    pub native_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Withdrawal {
    pub ready_at: Timestamp,
    pub denom_or_address: String,
    pub amount: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const WITHDRAWAL_READY: Item<Option<Withdrawal>> = Item::new("withdrawal_ready");

// a mapping of CW20 contract_address -> balance held by this contract
pub const CW20_BALANCES: Map<Addr, Uint128> = Map::new("cw20_balances");
