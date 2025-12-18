//! The storage contract ABI for the Arkiv network. Contains bindings to event data used in decoding event logs and receipts.

use alloy::{primitives::Address, rpc::types::Log, sol_types::SolEventInterface};

use crate::EntityKey;

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
            uint256 newExpirationBlock
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
            address newOwner
        );
    }
}

/// Represents a Arkiv event parsed from the blockchain log.
/// Used to distinguish between entity creation, update, and removal events.
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum ArkivEvent {
    /// Entity was created.
    /// Contains the entity ID, block number, and transaction hash.
    EntityCreated {
        /// The ID of the created entity
        entity_id: EntityKey,
        /// The expiration block of the entity
        expiration_block: u64,
        /// The block number where the event occurred
        block_number: u64,
        /// The transaction hash that triggered the event
        transaction_hash: EntityKey,
    },
    /// Entity was updated.
    /// Contains the entity ID, block number, and transaction hash.
    EntityUpdated {
        /// The ID of the updated entity
        entity_id: EntityKey,
        /// The expiration block of the entity
        expiration_block: u64,
        /// The block number where the event occurred
        block_number: u64,
        /// The transaction hash that triggered the event
        transaction_hash: EntityKey,
    },
    /// Entity was removed.
    /// Contains the entity ID, block number, and transaction hash.
    EntityRemoved {
        /// The ID of the removed entity
        entity_id: EntityKey,
        /// The block number where the event occurred
        block_number: u64,
        /// The transaction hash that triggered the event
        transaction_hash: EntityKey,
    },
    /// Entity was extended.
    /// Contains the entity ID, block number, and transaction hash.
    EntityExtended {
        /// The ID of the removed entity
        entity_id: EntityKey,
        /// The old expiration block
        old_expiration_block: u64,
        /// The new expiration block
        new_expiration_block: u64,
        /// The block number where the event occurred
        block_number: u64,
        /// The transaction hash that triggered the event
        transaction_hash: EntityKey,
    },
    EntityTransferred {
        // TODO: Add docstrings
        /// The ID of the removed entity
        entity_id: EntityKey,
        old_owner: Address,
        new_owner: Address,
        /// The block number where the event occurred
        block_number: u64,
        /// The transaction hash that triggered the event
        transaction_hash: EntityKey,
    },
}
impl TryFrom<Log> for ArkivEvent {
    // TODO: Remove anyhow
    type Error = anyhow::Error;

    /// Attempts to parse a blockchain log into a `Event`.
    /// Returns an error if required fields are missing or the event type is unknown.
    fn try_from(log: Log) -> Result<Self, Self::Error> {
        let block_number = log
            .block_number
            .ok_or_else(|| anyhow::anyhow!("Missing block number"))?;
        let transaction_hash = log
            .transaction_hash
            .ok_or_else(|| anyhow::anyhow!("Missing transaction hash"))?;
        let parsed = ArkivAbi::ArkivAbiEvents::decode_log(&log.into())?;
        match parsed.data {
            ArkivAbi::ArkivAbiEvents::EntityCreated(data) => Ok(ArkivEvent::EntityCreated {
                entity_id: data.entityKey.into(),
                expiration_block: data.expirationBlock.try_into().unwrap_or_default(),
                block_number,
                transaction_hash,
            }),
            ArkivAbi::ArkivAbiEvents::EntityUpdated(data) => Ok(ArkivEvent::EntityUpdated {
                entity_id: data.entityKey.into(),
                expiration_block: data.newExpirationBlock.try_into().unwrap_or_default(),
                block_number,
                transaction_hash,
            }),
            ArkivAbi::ArkivAbiEvents::EntityDeleted(data) => Ok(ArkivEvent::EntityRemoved {
                entity_id: data.entityKey.into(),
                block_number,
                transaction_hash,
            }),
            ArkivAbi::ArkivAbiEvents::EntityExtended(data) => Ok(ArkivEvent::EntityExtended {
                entity_id: data.entityKey.into(),
                old_expiration_block: data.oldExpirationBlock.try_into().unwrap_or_default(),
                new_expiration_block: data.newExpirationBlock.try_into().unwrap_or_default(),
                block_number,
                transaction_hash,
            }),
            ArkivAbi::ArkivAbiEvents::EntityTransferred(data) => {
                Ok(ArkivEvent::EntityTransferred {
                    entity_id: data.entityKey.into(),
                    old_owner: data.oldOwner,
                    new_owner: data.newOwner,
                    block_number,
                    transaction_hash,
                })
            }
        }
    }
}
