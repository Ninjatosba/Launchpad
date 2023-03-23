

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub batch_duration: Uint128,
    pub batch_amount: Uint128,
    pub revenue_collector: Addr,
    pub price: Decimal,
    pub buy_denom: String,
    pub sell_denom: Addr,
    pub first_batch_release_time: Timestamp,
}
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub enum Status {
    // Waiting
    Pending,
    // Sale has started
    Active,
    // Distribution has started
    Distribution,
}
// implement to_string for Status
impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::Pending => "pending".to_string(),
            Status::Active => "active".to_string(),
            Status::Distribution => "distribution".to_string(),
        }
    }
}

#[cw_serde]
pub struct State {
    pub total_revenue: Uint128,
    pub total_sold: Uint128,
    pub status: Status,
}
pub const STATE: Item<State> = Item::new("state");

#[cw_serde]
pub struct Batch {
    pub amount: Uint128,
    pub release_time: Timestamp,
    pub released: bool,
}
pub type Bathces = Vec<Batch>;

#[cw_serde]
pub struct Position {
    pub address: Addr,
    pub total_bought: Uint128,
    pub total_paid: Uint128,
    pub price: Decimal,
    pub timestamp: Timestamp,

    // vector of batches
    pub batches: Bathces,
}
pub const POSITIONS: Map<Addr, Position> = Map::new("positions");
