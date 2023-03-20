use thiserror::Error;
use cosmwasm_std::{ConversionOverflowError, DivideByZeroError, OverflowError, StdError,};
use cw_utils::PaymentError;
use std::convert::Infallible;


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
    SaleNotActive {}
}
