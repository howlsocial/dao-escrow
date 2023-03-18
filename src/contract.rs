#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::{
    execute_cw20_withdraw, execute_escrow_cw20_withdraw, execute_receive, execute_withdraw,
    override_withdraw, start_withdraw, update_override_address, update_withdrawal_address,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query::{
    get_withdraw_ready, get_withdraw_requested, query_config, query_withdraw_ready,
};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:dao-escrow";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let withdraw_address = deps.api.addr_validate(&msg.withdraw_address)?;
    let override_address = deps.api.addr_validate(&msg.override_address)?;

    let config = Config {
        override_address: override_address.clone(),
        withdraw_address: withdraw_address.clone(),
        set_override_as_immutable: msg.set_override_as_immutable,
        set_withdraw_as_immutable: msg.set_withdraw_as_immutable,
        withdraw_delay_in_days: msg.withdraw_delay_in_days,
        native_denom: msg.native_denom,
        enable_cw20_receive: msg.enable_cw20_receive,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("override_address", override_address)
        .add_attribute("withdraw_address", withdraw_address)
        .add_attribute("withdraw_delay", msg.withdraw_delay_in_days.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartWithdraw {
            denom_or_address,
            amount,
        } => start_withdraw(deps, env, info, denom_or_address, amount),
        ExecuteMsg::ExecuteNativeWithdraw { denom, amount } => {
            execute_withdraw(deps, env, info, denom, amount)
        }
        ExecuteMsg::ExecuteCW20Withdraw { address, amount } => {
            execute_cw20_withdraw(deps, env, info, address, amount)
        }
        ExecuteMsg::ExecuteEscrowCW20Withdraw { address, amount } => {
            execute_escrow_cw20_withdraw(deps, env, info, address, amount)
        }
        ExecuteMsg::OverrideWithdraw {} => override_withdraw(deps, env, info),
        ExecuteMsg::UpdateOverrideAddress { address } => {
            update_override_address(deps, env, info, address)
        }
        ExecuteMsg::UpdateWithdrawalAddress { address } => {
            update_withdrawal_address(deps, env, info, address)
        }
        ExecuteMsg::Receive(wrapped) => execute_receive(deps, env, info, wrapped),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetWithdrawalReadyTime {} => to_binary(&get_withdraw_ready(deps)?),
        QueryMsg::IsWithdrawalReady {} => to_binary(&query_withdraw_ready(deps, env)?),
        QueryMsg::GetWithdrawalRequested {} => to_binary(&get_withdraw_requested(deps, env)?),
    }
}
