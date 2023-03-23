#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw20::Cw20QueryMsg;
use cw_asset::Asset;
// use cw2::set_contract_version;
use crate::batch::{create_batches, update_batches};
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryConfigResponse, QueryMsg, QueryPositionResponse,
    QueryStateResponse,
};
use crate::state::{Batch, Bathces, Config, Position, State, Status, CONFIG, POSITIONS, STATE};
use cw_utils::{maybe_addr, must_pay};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = maybe_addr(deps.api, msg.admin)?.unwrap_or_else(|| info.sender.clone());

    let config = Config {
        admin,
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
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
    }
}

pub fn execute_buy(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    // Check if sale is active
    if state.status != Status::Active {
        return Err(ContractError::SaleNotActive {});
    }

    let amount_paid = must_pay(&info, &config.buy_denom)?;
    let buy_amount = Decimal::from_ratio(amount_paid, Uint128::from(1u128))
        .checked_mul(config.price)
        .unwrap();
    // floor buy_amount
    let buy_amount = buy_amount.to_uint_floor();
    let position = POSITIONS.may_load(deps.storage, info.sender.clone())?;
    let _position = match position {
        Some(mut position) => {
            // if position does exist, add buy_amount to total_bought and total_paid and update batches
            position.total_bought += buy_amount;
            position.total_paid += amount_paid;
            let new_batches = update_batches(position.batches, amount_paid, config.batch_amount)?;
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
                total_paid: amount_paid,
                price: config.price,
                timestamp: env.block.time,
                batches,
            }
        }
    };
    // Send revenue to revenue_collector
    let revenue_asset = Asset::native(config.buy_denom, amount_paid);
    let revenue_msg = revenue_asset.transfer_msg(config.revenue_collector)?;

    let res = Response::default()
        .add_message(revenue_msg)
        .add_attribute("action", "buy")
        .add_attribute("amount_paid", amount_paid)
        .add_attribute("buy_amount", buy_amount);

    Ok(res)
}
#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
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
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
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
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
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
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
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

    let res = Response::default()
        .add_attributes(vec![
            attr("action", "admin_withdraw"),
            attr("amount", amount.to_string()),
        ])
        .add_message(withdraw_msg);

    Ok(res)
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    if state.status != Status::Distribution {
        return Err(ContractError::SaleNotDistribution {});
    }
    let mut position = POSITIONS.load(deps.storage, info.sender.clone())?;
    // check if is there any batch to claim
    let mature_claims: Vec<Batch> = position
        .clone()
        .batches
        .into_iter()
        .filter(|batch| batch.release_time < env.block.time)
        .collect();

    if mature_claims.is_empty() {
        return Err(ContractError::NoMatureClaims {});
    }
    // remove mature claims from position
    position
        .batches
        .retain(|batch| batch.release_time >= env.block.time);

    // save position
    POSITIONS.save(deps.storage, info.sender.clone(), &position)?;

    // calculate total amount to claim
    let total_amount: Uint128 = mature_claims
        .into_iter()
        .map(|batch| batch.amount)
        .sum::<Uint128>();

    let claim_asset = Asset::cw20(config.sell_denom, total_amount);
    let claim_msg = claim_asset.transfer_msg(info.sender)?;

    let res = Response::default()
        .add_attributes(vec![
            attr("action", "claim"),
            attr("amount", total_amount.to_string()),
        ])
        .add_message(claim_msg);

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::QueryState {} => to_binary(&query_state(deps)?),
        QueryMsg::QueryPosition { address } => to_binary(&query_position(deps, address)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<QueryConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(QueryConfigResponse {
        admin: config.admin.to_string(),
        batch_duration: config.batch_duration,
        batch_amount: config.batch_amount,
        revenue_collector: config.revenue_collector.to_string(),
        price: config.price,
        buy_denom: config.buy_denom,
        sell_denom: config.sell_denom.to_string(),
        first_batch_release_time: config.first_batch_release_time,
    })
}

pub fn query_state(deps: Deps) -> StdResult<QueryStateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(QueryStateResponse {
        status: state.status.to_string(),
        total_sold: state.total_sold,
        total_revenue: state.total_revenue,
    })
}

pub fn query_position(deps: Deps, address: String) -> StdResult<QueryPositionResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let position = POSITIONS.load(deps.storage, addr)?;
    Ok(QueryPositionResponse {
        address: position.address.to_string(),
        total_bought: position.total_bought,
        total_paid: position.total_paid,
        price: position.price,
        timestamp: position.timestamp,
        batches: position.batches,
    })
}
