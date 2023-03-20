#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, attr, Uint128, Decimal};
// use cw2::set_contract_version;
use cw_utils::{maybe_addr, must_pay};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, self};
use crate::state::{Config, CONFIG, STATE, State, POSITIONS, Position, Status};


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

    let admin=
        maybe_addr(deps.api, msg.admin)?.unwrap_or_else(|| info.sender.clone());

    let config = Config {
        admin: admin,
        batch_duration: msg.batch_duration,
        batch_amount: msg.batch_amount,
        revenue_collector: msg.revenue_collector,
        price: msg.price,
        buy_denom: msg.buy_denom,
        sell_denom: msg.sell_denom,
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
        ExecuteMsg::StartDistribution {} => execute_start_distribution(deps, env, info),
        ExecuteMsg::AdminWithdraw {} => execute_admin_withdraw(deps, env, info),

        
    }

}

pub fn execute_buy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    let amount = must_pay(&info,&config.buy_denom)?;
    let buy_amount = Decimal::from_ratio(amount, Uint128::from(1u128)).checked_mul(config.price).unwrap();
    // floor buy_amount
    let buy_amount = Uint128::from(buy_amount.to_uint_floor());

    let position = POSITIONS.may_load(deps.storage, info.sender)?;
    let mut position = match position {
        Some(p) => p,
        None => Position {
            address: info.sender.to_string(),
            total_amount: Uint128::zero(),
            price: Decimal::zero(),
            timestamp: env.block.time,
            batches: vec![],
        },
    };

    let mut res = Response::default();
    res.attributes = vec![
        attr("action", "buy"),
        attr("amount", amount.to_string()),
    ];
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
