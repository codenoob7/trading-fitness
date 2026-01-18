//! Core trading fitness calculations in Rust.
//!
//! This crate provides high-performance implementations of ITH (Investment Time Horizon)
//! analysis and related fitness metrics.

pub mod ith;
pub mod metrics;
pub mod types;

pub use ith::*;
pub use metrics::*;
pub use types::*;
