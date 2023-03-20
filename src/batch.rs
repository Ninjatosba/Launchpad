use std::ops::Mul;

use cosmwasm_std::{Uint128, Timestamp};

use crate::{state::{Batch , Bathces}, ContractError};

pub fn create_batches(batch_duration:Uint128, batch_amount:Uint128, amount:Uint128, release_time:Timestamp) -> Result < Bathces ,  ContractError>{
    let mut batches: Bathces ;
    // Create batches as per batch_amount
    let amount_per_batch = amount.checked_div(batch_amount)?;
    for i in 0..batch_amount.into(){
        let duration_of_batch = batch_duration.mul(Uint128::from(i));
        let batch = Batch{
            amount: amount_per_batch,
            release_time: release_time.plus_nanos(duration_of_batch.u128().mul(i) as u64),
            released: false,
        };
        batches.push(batch);
    }


    Ok(batches)
}

