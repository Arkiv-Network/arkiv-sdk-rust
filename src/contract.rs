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
        event EntityCreated(
            uint256 indexed entityKey,
            uint256 expirationBlock
        );

        /// The event emitted from the storage contract when an entity's data is updated.
        ///
        event EntityUpdated(
            uint256 indexed entityKey,
            uint256 expirationBlock
        );

        /// The event emitted from the storage contract when an entity is deleted.
        ///
        event EntityDeleted(
            uint256 indexed entityKey
        );

        /// The event emitted from the storage contract when an entity's life ([`crate::BlocksToLive`]) is extended.
        ///
        event EntityExtended(
            uint256 indexed entityKey,
            uint256 oldExpirationBlock,
            uint256 newExpirationBlock
        );

        // TODO: Figure out if this is all that's necessary
        /// The event emitted from the storage contract when an entity is transferred from one address to another.
        ///
        event EntityTransferred(
            uint256 indexed entityKey,
            address oldOwner,
            address newOwner,
        );
    }
}
