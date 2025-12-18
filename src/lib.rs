#![doc = include_str!("../README.md")]

mod contract;
pub mod entity;
pub mod error;
pub mod network;
pub mod providers;
pub mod rpc;
pub mod tx;
pub mod utils;

pub use contract::STORAGE_ADDRESS;
pub use entity::{Attribute, BlocksToLive, ContentType, EntityKey};
pub use network::StorageNetwork;
pub use providers::StorageProvider;
pub use tx::{ops, receipt};

/// Re-export of commonly used types from `alloy`.
pub use alloy::{
    primitives::{Address, keccak256},
    providers::DynProvider,
    signers::{Signature, local::PrivateKeySigner},
    transports::http::reqwest::Url,
};
