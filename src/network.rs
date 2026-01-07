use alloy::network::{Ethereum, Network};

/// Captures type info for network-specific RPC requests/responses.
///
/// [`alloy::network` Documentation](https://github.com/alloy-rs/alloy/tree/f341fd655a2ae230adcefee3f39a55e7b87ab620/crates/network)
///
/// This trait maintains dynamic compatibility with [`alloy::providers::DynProvider`].
pub trait StorageNetwork: Network {
    /// The transaction type of the [`StorageNetwork`]. See [`alloy::network::TransactionRequest`].
    type StorageTransactionRequest: crate::tx::StorageTransactionBuilder<Self> + Send;

    /// An event type emitted from a storage contract that can be parsed from [`alloy::rpc::types::Log`]s.
    type StorageEvent: TryFrom<alloy::rpc::types::Log> + Send;
}

/// Implementation of the [`StorageNetwork`] trait for [`alloy::network::Ethereum`].
///
/// [`alloy::network` Documentation](https://github.com/alloy-rs/alloy/tree/f341fd655a2ae230adcefee3f39a55e7b87ab620/crates/network)
impl StorageNetwork for Ethereum {
    type StorageTransactionRequest = crate::tx::StorageTransactionRequest<Self>;
    type StorageEvent = crate::contract::ArkivEvent;
}
