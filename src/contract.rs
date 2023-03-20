#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, wasm_execute, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, WasmQuery,
};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg};
use cw_asset::Asset;
// use cw2::set_contract_version;
use crate::batch::{create_batches, update_batches};
use crate::error::ContractError;
use crate::msg::{self, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    self, Batch, Bathces, Config, Position, State, Status, CONFIG, POSITIONS, STATE,
};
use cw_utils::{maybe_addr, must_pay};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:launchpad";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = maybe_addr(deps.api, msg.admin)?.unwrap_or_else(|| info.sender.clone());

    let config = Config {
        admin: admin,
        batch_duration: msg.batch_duration,
        batch_amount: msg.batch_amount,
        revenue_collector: deps.api.addr_validate(&msg.revenue_collector)?,
        price: msg.price,
        buy_denom: msg.buy_denom,
        sell_denom: deps.api.addr_validate(&msg.sell_denom)?,
        first_batch_release_time: msg.first_batch_release_time,
    };

    let state = State {
        total_revenue: Uint128::zero(),
        total_sold: Uint128::zero(),
        // Set status to pending
        status: Status::Pending,
    };

    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;

    let mut res = Response::default();
    res.attributes = vec![
        attr("action", "instantiate"),
        attr("admin", config.admin),
        attr("batch_duration", config.batch_duration.to_string()),
        attr("batch_amount", config.batch_amount.to_string()),
        attr("revenue_collector", config.revenue_collector),
        attr("price", config.price.to_string()),
        attr("buy_denom", config.buy_denom),
        attr("sell_denom", config.sell_denom),
    ];
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Buy {} => execute_buy(deps, env, info),
        ExecuteMsg::UpdateConfig {
            admin,
            batch_duration,
            batch_amount,
            revenue_collector,
            price,
            buy_denom,
            sell_denom,
        } => execute_update_config(
            deps,
            env,
            info,
            admin,
            batch_duration,
            batch_amount,
            revenue_collector,
            price,
            buy_denom,
            sell_denom,
        ),
        ExecuteMsg::StartSale {} => execute_start_sale(deps, env, info),
        ExecuteMsg::StartDistribution {} => execute_start_distribution(deps, env, info),
        ExecuteMsg::AdminWithdraw { amount } => execute_admin_withdraw(deps, env, info, amount),
    }
}

pub fn execute_buy(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    // Check if sale is active
    if state.status != Status::Active {
        return Err(ContractError::SaleNotActive {});
    }

    let amount = must_pay(&info, &config.buy_denom)?;
    let buy_amount = Decimal::from_ratio(amount, Uint128::from(1u128))
        .checked_mul(config.price)
        .unwrap();
    // floor buy_amount
    let buy_amount = Uint128::from(buy_amount.to_uint_floor());
    let position = POSITIONS.may_load(deps.storage, info.sender.clone())?;
    let mut position = match position {
        Some(mut position) => {
            // if position does exist, add buy_amount to total_bought and total_paid and update batches
            position.total_bought += buy_amount;
            position.total_paid += amount;
            let new_batches = update_batches(position.batches, amount, config.batch_amount)?;
            position.batches = new_batches;
            position
        }

        None => {
            let batches: Bathces = create_batches(
                config.batch_duration,
                config.batch_amount,
                buy_amount,
                config.first_batch_release_time,
            )?;
            Position {
                address: info.sender,
                total_bought: buy_amount,
                total_paid: amount,
                price: config.price,
                timestamp: env.block.time,
                batches: batches,
            }
        }
    };

    let mut res = Response::default();
    res.attributes = vec![attr("action", "buy"), attr("amount", amount.to_string())];
    Ok(res)
}

pub fn execute_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    batch_duration: Option<Uint128>,
    batch_amount: Option<Uint128>,
    revenue_collector: Option<String>,
    price: Option<Decimal>,
    buy_denom: Option<String>,
    sell_denom: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    if let Some(admin) = admin {
        config.admin = deps.api.addr_validate(&admin)?;
    }
    if let Some(batch_duration) = batch_duration {
        if state.status == Status::Pending {
            config.batch_duration = batch_duration;
        } else {
            return Err(ContractError::SaleNotPending {});
        }
    }
    if let Some(batch_amount) = batch_amount {
        if state.status == Status::Pending {
            config.batch_amount = batch_amount;
        } else {
            return Err(ContractError::SaleNotPending {});
        }
    }
    if let Some(revenue_collector) = revenue_collector {
        config.revenue_collector = deps.api.addr_validate(&revenue_collector)?;
    }
    if let Some(price) = price {
        config.price = price;
    }
    if let Some(buy_denom) = buy_denom {
        if state.status == Status::Pending {
            config.buy_denom = buy_denom;
        } else {
            return Err(ContractError::SaleNotPending {});
        }
    }
    if let Some(sell_denom) = sell_denom {
        if state.status == Status::Pending {
            config.sell_denom = deps.api.addr_validate(&sell_denom)?;
        } else {
            return Err(ContractError::SaleNotPending {});
        }
    }

    CONFIG.save(deps.storage, &config)?;
    let mut res = Response::default();
    res.attributes = vec![
        attr("action", "update_config"),
        attr("admin", config.admin),
        attr("batch_duration", config.batch_duration.to_string()),
        attr("batch_amount", config.batch_amount.to_string()),
        attr("revenue_collector", config.revenue_collector),
        attr("price", config.price.to_string()),
        attr("buy_denom", config.buy_denom),
        attr("sell_denom", config.sell_denom),
    ];
    Ok(res)
}

pub fn execute_start_sale(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    if state.status != Status::Pending {
        return Err(ContractError::SaleNotPending {});
    }
    state.status = Status::Active;
    STATE.save(deps.storage, &state)?;
    let mut res = Response::default();
    res.attributes = vec![attr("action", "start_sale")];
    Ok(res)
}

pub fn execute_start_distribution(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    if state.status != Status::Active {
        return Err(ContractError::SaleNotActive {});
    }
    state.status = Status::Distribution;
    STATE.save(deps.storage, &state)?;
    let mut res = Response::default();
    res.attributes = vec![attr("action", "start_distribution")];
    Ok(res)
}

pub fn execute_admin_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    if state.status != Status::Distribution {
        return Err(ContractError::SaleNotDistribution {});
    }
    // check cw20 contract balance
    let cw20_balance: Uint128 = deps.querier.query_wasm_smart(
        config.sell_denom.clone(),
        &Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    if cw20_balance < amount {
        return Err(ContractError::InsufficientBalance {});
    }

    let withdraw_asset = Asset::cw20(config.sell_denom, amount);
    let withdraw_msg = withdraw_asset.transfer_msg(info.sender)?;
    // refactor bellow

    let mut res = Response::default()
        .add_attributes(vec![
            attr("action", "admin_withdraw"),
            attr("amount", amount.to_string()),
        ])
        .add_message(withdraw_msg);

    Ok(res)
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
