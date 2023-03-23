use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Timestamp, Uint128};

use crate::state::Batch;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub batch_duration: Uint128,
    pub batch_amount: Uint128,
    pub revenue_collector: String,
    pub price: Decimal,
    // In this case buy_denom is native token
    pub buy_denom: String,
    // In this case sell_denom is cw20 token
    pub sell_denom: String,
    // First batch release time. This shouldnt be changed once sale is started.
    pub first_batch_release_time: Timestamp,
}

#[cw_serde]
pub enum ExecuteMsg {
    Buy {},
    UpdateConfig {
        admin: Option<String>,
        batch_duration: Option<Uint128>,
        batch_amount: Option<Uint128>,
        revenue_collector: Option<String>,
        price: Option<Decimal>,
        buy_denom: Option<String>,
        sell_denom: Option<String>,
    },
    StartDistribution {},
    // Withdraw remaning cw20 tokens. Checks balance and sends remaining tokens to admin
    AdminWithdraw {
        amount: Uint128,
    },
    StartSale {},
    Claim {},
}

#[cw_serde]
pub enum QueryMsg {
    QueryPosition { address: String },
    QueryConfig {},
    QueryState {},
}
#[cw_serde]
pub struct QueryPositionResponse {
    pub address: String,
    pub total_bought: Uint128,
    pub total_paid: Uint128,
    pub price: Decimal,
    pub timestamp: Timestamp,
    pub batches: Vec<Batch>,
}
#[cw_serde]
pub struct QueryConfigResponse {
    pub admin: String,
    pub batch_duration: Uint128,
    pub batch_amount: Uint128,
    pub revenue_collector: String,
    pub price: Decimal,
    pub buy_denom: String,
    pub sell_denom: String,
    pub first_batch_release_time: Timestamp,
}
#[cw_serde]
pub struct QueryStateResponse {
    pub total_revenue: Uint128,
    pub total_sold: Uint128,
    pub status: String,
}
