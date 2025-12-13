#![doc = include_str!("../README.md")]

pub mod entity;
pub mod error;
pub mod eth;
pub mod events;
pub mod network;
pub mod providers;
pub mod rpc;
pub mod tx;
pub mod utils;

pub use entity::{EntityKey, attribute::Attribute};

/// Re-export of commonly used types from `alloy`.
pub use alloy::{
    primitives::{Address, keccak256},
    providers::DynProvider,
    signers::{Signature, local::PrivateKeySigner},
    transports::http::reqwest::Url,
};
