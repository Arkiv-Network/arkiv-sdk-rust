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
    alloy::primitives::address!("0x0000000000000000000000000000000060138453");

alloy::sol! {
    /// The storage contract ABI for the Arkiv network. Contains bindings to event data used in decoding event logs and receipts.
    contract ArkivAbi {
        /// The event emitted from the storage contract when an entity is created.
        ///
        event ArkivEntityCreated(
            uint256 indexed entityKey,
            uint256 expirationBlock
        );

        /// The event emitted from the storage contract when an entity's data is updated.
        ///
        event ArkivEntityUpdated(
            uint256 indexed entityKey,
            uint256 newExpirationBlock
        );

        /// The event emitted from the storage contract when an entity is deleted.
        ///
        event ArkivEntityDeleted(
            uint256 indexed entityKey
        );

        /// The event emitted from the storage contract when an entity's life ([`crate::BlocksToLive`]) is extended.
        ///
        event ArkivEntityBTLExtended(
            uint256 indexed entityKey,
            uint256 oldExpirationBlock,
            uint256 newExpirationBlock
        );

        /// The event emitted from the storage contract when an entity is transferred from one address to another.
        ///
        event ArkivEntityOwnerChanged(
            uint256 indexed entityKey,
            address indexed oldOwner,
            address indexed newOwner
        );

        /// The event emitted when an entity is automatically removed by the housekeeping system due to expiration.
        ///
        event ArkivEntityExpired(
            uint256 indexed entityKey,
            address indexed ownerAddress
        );
    }
}
