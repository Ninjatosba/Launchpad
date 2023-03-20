use cosmwasm_std::{ConversionOverflowError, DivideByZeroError, OverflowError, StdError};
use cw_asset::AssetError;
use cw_utils::PaymentError;
use std::convert::Infallible;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Infallible(#[from] Infallible),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("Sale is not active")]
    SaleNotActive {},

    #[error("Sale can not be started because it is either active or distribution")]
    SaleNotPending {},

    #[error("Sale is not in distribution state")]
    SaleNotDistribution {},

    #[error("Insufficient balance")]
    InsufficientBalance {},
    #[error("Asset error")]
    AssetError {},
}

impl From<AssetError> for ContractError {
    fn from(_err: AssetError) -> Self {
        ContractError::AssetError {}
    }
}
