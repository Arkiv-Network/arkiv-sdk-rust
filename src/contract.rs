//! The storage contract ABI for the Arkiv network. Contains bindings to event data used in decoding event logs and receipts.

mod error;
pub use error::Error;

/// The Ethereum address of the Arkiv storage contract. All entity-related transactions are sent to this address.
///
/// This address does not need to be provided when sending a transaction. The [`crate::tx::StorageTransactionBuilder`]
/// trait implementation for [`crate::tx::StorageTransactionRequest`] uses this address internally.
///
/// For convenience, this address can be referred to directly from the [`crate::tx::StorageTransactionRequest`] itself.
pub const STORAGE_ADDRESS: alloy::primitives::Address =
    alloy::primitives::address!("0x00000000000000000000000000000061726B6976");

alloy::sol! {
    /// The storage contract ABI for the Arkiv network. Contains bindings to event data used in decoding event logs and receipts.
    /// The version of this contract is taken directly from this commit: <https://github.com/Arkiv-Network/arkiv-op-geth/blob/4d831af4042037469d9b941e41cd79d78fd37f4c/arkiv/logs/arkiv.go>.
    contract ArkivAbi {
        /// ArkivEntityCreated is the event signature for entity creation logs.
        ///
        /// Parameters: entityKey (indexed), ownerAddress(indexed), expirationBlock, cost (wei)
        ///
        event ArkivEntityCreated(
            uint256 indexed entityKey,
            address indexed ownerAddress,
            uint256 expirationBlock,
            uint256 cost,
        );

        /// ArkivEntityUpdated is the event signature for entity update logs.
        ///
        /// Parameters: entityKey (indexed), ownerAddress(indexed), oldExpirationBlock, newExpirationBlock, cost (wei)
        ///
        event ArkivEntityUpdated(
            uint256 indexed entityKey,
            address indexed ownerAddress,
            uint256 oldExpirationBlock,
            uint256 newExpirationBlock,
            uint256 cost,
        );

        /// ArkivEntityExpired is the event signature for entity expiration logs.
        ///
        /// Parameters: entityKey (indexed), ownerAddress(indexed)
        ///
        event ArkivEntityExpired(
            uint256 indexed entityKey,
            address indexed ownerAddress,
        );

        /// ArkivEntityDeleted is the event signature for entity deletion logs.
        ///
        /// Parameters: entityKey (indexed), ownerAddress(indexed)
        ///
        event ArkivEntityDeleted(
            uint256 indexed entityKey,
            address indexed ownerAddress,
        );

        /// ArkivEntityBTLExtended is the event signature for extending BTL of an entity.
        ///
        /// Parameters: entityKey (indexed), ownerAddress(indexed), oldExpirationBlock, newExpirationBlock, cost (wei)
        ///
        event ArkivEntityBTLExtended(
            uint256 indexed entityKey,
            address indexed ownerAddress,
            uint256 oldExpirationBlock,
            uint256 newExpirationBlock,
            uint256 cost,
        );

        /// ArkivEntityOwnerChanged is the event signature for changing the owner of an entity.
        ///
        /// Parameters: entityKey (indexed), oldOwnerAddress(indexed), newOwnerAddress(indexed)
        ///
        event ArkivEntityOwnerChanged(
            uint256 indexed entityKey,
            address indexed oldOwnerAddress,
            address indexed newOwnerAddress,
        );
    }
}
