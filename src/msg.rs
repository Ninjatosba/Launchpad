use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub batch_duration: u64,
    pub batch_amount: u64,
    pub revenue_collector: String,
    pub price: Decimal,
    // In this case buy_denom is native token
    pub buy_denom: String,
    // In this case sell_denom is cw20 token
    pub sell_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Buy {},
    UpdateConfig {
        admin: Option<String>,
        batch_duration: Option<u64>,
        batch_amount: Option<u64>,
        revenue_collector: Option<String>,
        price: Option<Decimal>,
        buy_denom: Option<String>,
        sell_denom: Option<String>,
    },
    StartDistribution {},
    // withdraw remaning cw20 tokens. Checks balance and sends remaining tokens to admin
    AdminWithdraw{
    }
    
    
    }

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
