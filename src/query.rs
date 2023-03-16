use cosmwasm_std::{Deps, Env, StdError, StdResult};

use crate::msg::{
    WithdrawalReadyResponse, WithdrawalRequestedResponse, WithdrawalTimestampResponse,
};
use crate::state::{Config, CONFIG, WITHDRAWAL_READY};

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn get_withdraw_ready(deps: Deps) -> StdResult<WithdrawalTimestampResponse> {
    let withdrawal_ready = WITHDRAWAL_READY.load(deps.storage)?;

    match withdrawal_ready {
        Some(wr) => Ok(WithdrawalTimestampResponse {
            withdrawal_ready_timestamp: wr.ready_at,
        }),
        None => Err(StdError::not_found(
            "Withdrawal not yet requested - no Withdrawal time exists",
        )),
    }
}

pub fn query_withdraw_ready(deps: Deps, env: Env) -> StdResult<WithdrawalReadyResponse> {
    let withdrawal_ready = WITHDRAWAL_READY.load(deps.storage)?;

    match withdrawal_ready {
        Some(wr) => {
            // check if we are have passed the point where withdrawal is possible
            let is_withdrawal_ready = env.block.time > wr.ready_at;

            Ok(WithdrawalReadyResponse {
                is_withdrawal_ready,
            })
        }
        None => Err(StdError::not_found(
            "Withdrawal not yet requested - no Withdrawal time exists",
        )),
    }
}

pub fn get_withdraw_requested(deps: Deps, _env: Env) -> StdResult<WithdrawalRequestedResponse> {
    // we catch an error as None to avoid double wrapping with may_load
    let withdrawal_ready = WITHDRAWAL_READY.load(deps.storage).unwrap_or(None);

    match withdrawal_ready {
        Some(_wr) => Ok(WithdrawalRequestedResponse {
            withdrawal_requested: true,
        }),
        None => Ok(WithdrawalRequestedResponse {
            withdrawal_requested: false,
        }),
    }
}
