use std::ops::Add;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Decimal256, Storage, Timestamp, Uint128, Uint64};
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
