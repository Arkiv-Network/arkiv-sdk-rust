#![doc = include_str!("../README.md")]

pub mod client;
pub mod entity;
pub mod error;
pub mod eth;
pub mod events;
pub mod rpc;
pub mod tx;
pub mod utils;

pub use client::{Client, RoClient};
pub use entity::{EntityKey, attribute::Attribute};

/// Re-export of commonly used types from `alloy`.
pub use alloy::{
    primitives::{Address, keccak256},
    signers::{Signature, local::PrivateKeySigner},
    transports::http::reqwest::Url,
};
