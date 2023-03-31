use cosmwasm_std::{
    ensure_eq, to_binary, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Timestamp,
    Uint128, WasmMsg,
};

use crate::error::ContractError;
use crate::state::{Config, Withdrawal, CONFIG, CW20_BALANCES, WITHDRAWAL_READY};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

// receive CW20 tokens
pub fn execute_receive(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    wrapped: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // check that config is set to true
    let config = CONFIG.load(deps.storage)?;
    let receive_enabled = config.enable_cw20_receive;
    ensure_eq!(receive_enabled, true, ContractError::CW20ReceiveDisabled {});

    // wrapped.sender is the CW20 contract sending tokens
    let cw20_addr = deps.api.addr_validate(&wrapped.sender)?;
    // wrapped.amount is the balance sent
    // we add to any balance that already exists and save
    let existing_balance = CW20_BALANCES.may_load(deps.storage, cw20_addr.clone())?;

    // calc the balance
    let updated_balance = match existing_balance {
        Some(balance) => {
            let new_balance: Uint128 = balance
                .checked_add(wrapped.amount)
                .map_err(|_| ContractError::CW20BalanceError {})?;
            new_balance
        }
        None => wrapped.amount,
    };

    // update that bad boy
    CW20_BALANCES.save(deps.storage, cw20_addr, &updated_balance)?;

    Ok(Response::new()
        .add_attribute("method", "receive")
        .add_attribute("balance", updated_balance))
}

// this sets the withdraw delay
// note that it does not withdraw funds immediately
pub fn start_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom_or_address: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // get config
    let config = CONFIG.load(deps.storage)?;
    let withdraw_address = config.withdraw_address;

    // before continuing, only withdraw_address can call this
    ensure_eq!(
        info.sender,
        withdraw_address,
        ContractError::Unauthorized {}
    );

    // get number of days delay
    let delay_in_days: u64 = config.withdraw_delay_in_days;

    // do some really simple maths
    let seconds_in_day = 86400u64;
    let delay_in_seconds = delay_in_days * seconds_in_day;

    // when is 'now'?
    let now: Timestamp = env.block.time;

    // calculate now + configured days (in seconds)
    let rewards_ready_at: Timestamp = now.plus_seconds(delay_in_seconds);

    let withdrawal = Withdrawal {
        ready_at: rewards_ready_at,
        denom_or_address,
        amount,
    };

    WITHDRAWAL_READY.save(deps.storage, &Some(withdrawal))?;

    Ok(Response::new()
        .add_attribute("action", "start_withdraw")
        .add_attribute("withdrawal_ready_timestamp", rewards_ready_at.to_string()))
}

// this allows you to withdraw if the withdraw delay has passed
pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
) -> Result<Response, ContractError> {
    // get withdraw address
    let config = CONFIG.load(deps.storage)?;
    let withdraw_address = config.withdraw_address;

    // before continuing, only withdraw_address can call this
    ensure_eq!(
        info.sender,
        withdraw_address,
        ContractError::Unauthorized {}
    );

    // get rewards ready timestamp
    let withdrawal_ready = WITHDRAWAL_READY.load(deps.storage)?;

    if let Some(wr) = withdrawal_ready {
        // check if we are after that time
        let withdrawal_claimable = env.block.time > wr.ready_at;

        // dispatch Response or ContractError
        match withdrawal_claimable {
            true => {
                // check this is what we expect
                ensure_eq!(
                    denom,
                    wr.denom_or_address,
                    ContractError::WithdrawalDenomMismatch {}
                );

                // set up a bank send to the withdraw address
                // from this contract
                // for the amount
                let msgs: Vec<CosmosMsg> = vec![BankMsg::Send {
                    to_address: withdraw_address.to_string(),
                    amount: vec![Coin {
                        denom,
                        amount: wr.amount,
                    }],
                }
                .into()];

                // reset the timer now we've claimed the withdrawal
                WITHDRAWAL_READY.save(deps.storage, &None)?;

                Ok(Response::new()
                    .add_attribute("action", "execute_withdraw")
                    .add_attribute("withdraw_address", withdraw_address)
                    .add_messages(msgs))
            }
            false => Err(ContractError::WithdrawalNotReady {}),
        }
    } else {
        Err(ContractError::WithdrawalNotRequested {})
    }
}

// this calls the address passed in, a CW20 contract,
// and gets it to transfer the balance assigned to this contract
// to the withdraw_address
pub fn execute_cw20_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    // get withdraw address
    let config = CONFIG.load(deps.storage)?;
    let withdraw_address = config.withdraw_address;

    // before continuing, only withdraw_address can call this
    ensure_eq!(
        info.sender,
        withdraw_address,
        ContractError::Unauthorized {}
    );

    // check the address we've been passed is kosher
    let validated_cw20_addr = deps.api.addr_validate(&address)?;

    // now we can get rewards ready timestamp
    // and see if we can send those tasty tasty cw20s
    let withdrawal_ready = WITHDRAWAL_READY.load(deps.storage)?;

    if let Some(wr) = withdrawal_ready {
        // check if we are after that time
        let withdrawal_claimable = env.block.time > wr.ready_at;

        // dispatch Response or ContractError
        match withdrawal_claimable {
            true => {
                // check the cw20 addr matches since this is denom
                let validated_requested_cw20_addr = deps.api.addr_validate(&wr.denom_or_address)?;
                ensure_eq!(
                    validated_cw20_addr,
                    validated_requested_cw20_addr,
                    ContractError::WithdrawalCW20Mismatch {}
                );

                // put together msg
                let msg = WasmMsg::Execute {
                    contract_addr: validated_cw20_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: withdraw_address.to_string(),
                        amount: wr.amount,
                    })?,
                    funds: vec![],
                };

                // reset the timer now we've claimed the withdrawal
                WITHDRAWAL_READY.save(deps.storage, &None)?;

                Ok(Response::new()
                    .add_attribute("action", "execute_withdraw")
                    .add_attribute("withdraw_address", withdraw_address)
                    .add_message(msg))
            }
            false => Err(ContractError::WithdrawalNotReady {}),
        }
    } else {
        Err(ContractError::WithdrawalNotRequested {})
    }
}

// this looks up a CW20 balance in the balances map
// if found, it clears the balance, and sends to the nominated address
// by calling the CW20 contract in question
pub fn execute_escrow_cw20_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    // get withdraw address
    let config = CONFIG.load(deps.storage)?;
    let withdraw_address = config.withdraw_address;

    // before continuing, only withdraw_address can call this
    ensure_eq!(
        info.sender,
        withdraw_address,
        ContractError::Unauthorized {}
    );

    // check that config is set to true
    let receive_enabled = config.enable_cw20_receive;
    ensure_eq!(receive_enabled, true, ContractError::CW20ReceiveDisabled {});

    // check the address we've been passed is kosher
    let validated_cw20_addr = deps.api.addr_validate(&address)?;

    // check if there's any balance
    let cw20_balance = CW20_BALANCES.load(deps.storage, validated_cw20_addr.clone())?;

    // now we can get rewards ready timestamp
    // and see if we can send those tasty tasty cw20s
    let withdrawal_ready = WITHDRAWAL_READY.load(deps.storage)?;

    if let Some(wr) = withdrawal_ready {
        // check if we are after that time
        let withdrawal_claimable = env.block.time > wr.ready_at;

        // dispatch Response or ContractError
        match withdrawal_claimable {
            true => {
                // ok let's check this is what we expect before continuing
                // this is effectively denom
                let validated_requested_cw20_addr = deps.api.addr_validate(&wr.denom_or_address)?;
                ensure_eq!(
                    validated_cw20_addr,
                    validated_requested_cw20_addr,
                    ContractError::WithdrawalCW20Mismatch {}
                );

                // first, find new balance
                let new_balance = cw20_balance
                    .checked_sub(wr.amount)
                    .map_err(|_| ContractError::CW20BalanceError {})?;

                // call the cw20 and transfer the balance to withdraw_address
                let msg = WasmMsg::Execute {
                    contract_addr: validated_cw20_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: withdraw_address.to_string(),
                        amount: wr.amount,
                    })?,
                    funds: vec![],
                };

                // then subtract from our internal treasury
                CW20_BALANCES.save(deps.storage, validated_cw20_addr, &new_balance)?;

                // reset the timer now we've claimed the withdrawal
                WITHDRAWAL_READY.save(deps.storage, &None)?;

                Ok(Response::new()
                    .add_attribute("action", "execute_withdraw")
                    .add_attribute("withdraw_address", withdraw_address)
                    .add_message(msg))
            }
            false => Err(ContractError::WithdrawalNotReady {}),
        }
    } else {
        Err(ContractError::WithdrawalNotRequested {})
    }
}

pub fn override_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // get override address
    let config = CONFIG.load(deps.storage)?;
    let override_address = config.override_address;

    // explicitly crash out if no withdrawal exists
    let withdrawal = WITHDRAWAL_READY.may_load(deps.storage)?;
    if withdrawal.is_none() {
        return Err(ContractError::WithdrawalNotRequested {});
    }

    // before continuing, only override_address can call this
    ensure_eq!(
        info.sender,
        override_address,
        ContractError::Unauthorized {}
    );

    WITHDRAWAL_READY.save(deps.storage, &None)?;

    Ok(Response::new().add_attribute("action", "override_withdraw"))
}

pub fn update_override_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    // get override address
    let config = CONFIG.load(deps.storage)?;
    let override_address = config.override_address;

    // before continuing, only override_address can call this
    ensure_eq!(
        info.sender,
        override_address,
        ContractError::Unauthorized {}
    );

    // then check config
    ensure_eq!(
        config.set_override_as_immutable,
        false,
        ContractError::OverrideAddressIsImmutable {}
    );

    let new_override_address = deps.api.addr_validate(&address)?;

    // update
    let new_config = Config {
        override_address: new_override_address.clone(),
        withdraw_address: config.withdraw_address,
        withdraw_delay_in_days: config.withdraw_delay_in_days,
        native_denom: config.native_denom,
        set_withdraw_as_immutable: config.set_withdraw_as_immutable,
        set_override_as_immutable: config.set_override_as_immutable,
        enable_cw20_receive: config.enable_cw20_receive,
    };

    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new()
        .add_attribute("action", "update_override_address")
        .add_attribute("new_override_address", new_override_address))
}

pub fn update_withdrawal_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    // get override address
    let config = CONFIG.load(deps.storage)?;
    let override_address = config.override_address;

    // before continuing, only override_address can call this
    ensure_eq!(
        info.sender,
        override_address,
        ContractError::Unauthorized {}
    );

    // but wait! can this even be changed?
    let is_immutable = config.set_withdraw_as_immutable;
    // if the above is false, then continue
    // otherwise error
    ensure_eq!(
        is_immutable,
        false,
        ContractError::WithdrawalAddressIsImmutable {}
    );

    let new_withdraw_address = deps.api.addr_validate(&address)?;

    // LFG, change it
    let new_config = Config {
        override_address,
        withdraw_address: new_withdraw_address.clone(),
        withdraw_delay_in_days: config.withdraw_delay_in_days,
        native_denom: config.native_denom,
        set_withdraw_as_immutable: config.set_withdraw_as_immutable,
        set_override_as_immutable: config.set_override_as_immutable,
        enable_cw20_receive: config.enable_cw20_receive,
    };

    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new()
        .add_attribute("action", "update_withdrawal_address")
        .add_attribute("new_withdraw_address", new_withdraw_address))
}
