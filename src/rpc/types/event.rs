#![cfg(feature = "pubsub")]
use alloy::{primitives::Address, rpc::types::Log, sol_types::SolEventInterface};

use crate::{EntityKey, contract::ArkivAbi};

/// Represents a Arkiv event parsed from the blockchain log.
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum ArkivEvent {
    /// Entity was created.
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
    EntityRemoved {
        /// The ID of the removed entity
        entity_id: EntityKey,
        /// The block number where the event occurred
        block_number: u64,
        /// The transaction hash that triggered the event
        transaction_hash: EntityKey,
    },
    /// Entity BTL was extended.
    EntityExtended {
        /// The ID of the extended entity
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
    /// Entity ownership was transferred.
    EntityTransferred {
        /// The ID of the removed entity
        entity_id: EntityKey,
        /// The previous owner's address
        old_owner: Address,
        /// The new owner's address
        new_owner: Address,
        /// The block number where the event occurred
        block_number: u64,
        /// The transaction hash that triggered the event
        transaction_hash: EntityKey,
    },
    /// A house-keeping event emitted when an entity expires.
    EntityExpired {
        /// The ID of the expired entity
        entity_id: EntityKey,
        /// The owner of the expired entity
        owner_address: Address,
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
            ArkivAbi::ArkivAbiEvents::ArkivEntityCreated(data) => Ok(ArkivEvent::EntityCreated {
                entity_id: data.entityKey.into(),
                expiration_block: data.expirationBlock.try_into().unwrap_or_default(),
                block_number,
                transaction_hash,
            }),
            ArkivAbi::ArkivAbiEvents::ArkivEntityUpdated(data) => Ok(ArkivEvent::EntityUpdated {
                entity_id: data.entityKey.into(),
                expiration_block: data.newExpirationBlock.try_into().unwrap_or_default(),
                block_number,
                transaction_hash,
            }),
            ArkivAbi::ArkivAbiEvents::ArkivEntityDeleted(data) => Ok(ArkivEvent::EntityRemoved {
                entity_id: data.entityKey.into(),
                block_number,
                transaction_hash,
            }),
            ArkivAbi::ArkivAbiEvents::ArkivEntityBTLExtended(data) => {
                Ok(ArkivEvent::EntityExtended {
                    entity_id: data.entityKey.into(),
                    old_expiration_block: data.oldExpirationBlock.try_into().unwrap_or_default(),
                    new_expiration_block: data.newExpirationBlock.try_into().unwrap_or_default(),
                    block_number,
                    transaction_hash,
                })
            }
            ArkivAbi::ArkivAbiEvents::ArkivEntityOwnerChanged(data) => {
                Ok(ArkivEvent::EntityTransferred {
                    entity_id: data.entityKey.into(),
                    old_owner: data.oldOwner,
                    new_owner: data.newOwner,
                    block_number,
                    transaction_hash,
                })
            }
            ArkivAbi::ArkivAbiEvents::ArkivEntityExpired(data) => Ok(ArkivEvent::EntityExpired {
                entity_id: data.entityKey.into(),
                owner_address: data.ownerAddress,
                block_number,
                transaction_hash,
            }),
        }
    }
}
