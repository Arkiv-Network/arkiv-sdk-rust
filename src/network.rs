// TODO: Add docstrings
use alloy::network::{Ethereum, Network};

pub trait StorageNetwork: Network {
    type StorageTransactionRequest: crate::tx::StorageTransactionBuilder<Self> + Send;
    type StorageEvent: TryFrom<alloy::rpc::types::Log> + Send;
}
impl StorageNetwork for Ethereum {
    type StorageTransactionRequest = crate::tx::StorageTransactionRequest<Self>;
    type StorageEvent = crate::contract::ArkivEvent;
}
