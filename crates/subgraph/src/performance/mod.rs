use rain_math_float::FloatError;
use raindex_math::MathError;
use std::num::ParseIntError;
use thiserror::Error;

// TODO: APY related logic
// use alloy::primitives::{ruint::ParseError, U256};
// use chrono::TimeDelta;
// use once_cell::sync::Lazy;
// use raindex_math::ONE18;

// TODO: APY related logic
// pub mod apy;
// mod order_performance;
pub mod vol;

// TODO: APY related logic
// pub use order_performance::*;

// TODO: APY related logic
// /// a year length timestamp in seconds as 18 point decimals as U256
// pub static YEAR18: Lazy<U256> =
//     Lazy::new(|| U256::from(TimeDelta::days(365).num_seconds()).saturating_mul(ONE18));

#[derive(Error, Debug)]
pub enum PerformanceError {
    #[error(transparent)]
    MathError(#[from] MathError),
    #[error("Float error: {0}")]
    FloatError(#[from] FloatError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("divide by zero")]
    DivByZero,
    #[error("Missing decimals in subgraph response")]
    MissingDecimals,
    // TODO: APY related logic
    // #[error(transparent)]
    // ParseUnsignedError(#[from] ParseError),
    // #[error("Found no trades")]
    // NoTrades,
    // #[error("Parsing error: {0}")]
    // ParsingError(#[from] serde_json::Error),
}
