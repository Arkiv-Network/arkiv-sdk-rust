#![doc = include_str!("../README.md")]

mod contract;
pub mod entity;
pub mod error;
pub mod network;
pub mod node_bindings;
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
    primitives::{keccak256, Address},
    providers::{DynProvider, Provider, ProviderBuilder},
    signers::{local::PrivateKeySigner, Signature},
    transports::http::reqwest::Url,
};
