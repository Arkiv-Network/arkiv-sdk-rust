use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{BlocksToLive, entity::EntityKey};

/// Type representing an extend operation as part of a [`crate::network::StorageNetwork::StorageTransactionRequest`].
/// Used to extend the [`crate::entity::BlocksToLive`] of an entity.
#[derive(Debug, Clone, Copy, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct Extend {
    /// The key of the entity to extend.
    entity_key: EntityKey,
    /// The number of blocks to extend the BTL by.
    number_of_blocks: u64,
}

impl Extend {
    /// Construct a new instance of an extend operation as part of a [`crate::network::StorageNetwork::StorageTransactionRequest`]
    /// for some existing [`crate::entity::Entity`] to extend the entity's [`crate::BlocksToLive`].
    /// The entity's resulting BTL will be the sum of the two [`crate::BlocksToLive`].
    pub fn new<K: Into<EntityKey>, B: Into<BlocksToLive>>(entity_key: K, btl: B) -> Self {
        Self {
            entity_key: entity_key.into(),
            number_of_blocks: btl.into().into(),
        }
    }

    /// The key of the entity to extend.
    pub fn entity_key(&self) -> EntityKey {
        self.entity_key
    }

    /// The number of blocks to extend the BTL by.
    pub fn number_of_blocks(&self) -> u64 {
        self.number_of_blocks
    }
}
