use std::ops::{Add, Mul};

use cosmwasm_std::{Timestamp, Uint128};

use crate::{
    state::{Batch, Bathces},
    ContractError,
};

pub fn create_batches(
    batch_duration: Uint128,
    batch_amount: Uint128,
    amount: Uint128,
    release_time: Timestamp,
) -> Result<Bathces, ContractError> {
    let mut batches: Bathces = vec![];
    // Create batches as per batch_amount
    let amount_per_batch = amount.checked_div(batch_amount)?;
    for i in 0..batch_amount.into() {
        let duration_of_batch = batch_duration.mul(Uint128::from(i));
        let batch = Batch {
            amount: amount_per_batch,
            release_time: release_time.plus_nanos(duration_of_batch.u128().mul(i) as u64),
            released: false,
        };
        batches.push(batch);
    }

    Ok(batches)
}

pub fn update_batches(
    batches: Bathces,
    amount: Uint128,
    batch_amount: Uint128,
) -> Result<Bathces, ContractError> {
    let new_batces = batches
        .iter()
        .map(|batch| {
            let new_batch = Batch {
                amount: batch.amount.add(amount / batch_amount),
                release_time: batch.release_time,
                released: false,
            };
            new_batch
        })
        .collect();
    Ok(new_batces)
}
