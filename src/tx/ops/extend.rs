use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::entity::EntityKey;

/// Type representing an extend operation as part of a `Transaction`.
/// Used to extend the [`crate::entity::BlocksToLive`] of an entity.
#[derive(Debug, Clone, Copy, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct Extend {
    /// The key of the entity to extend.
    entity_key: EntityKey,
    /// The number of blocks to extend the BTL by.
    number_of_blocks: u64,
}

impl Extend {
    /// Construct a new instance of an extend operation as part of a `Transaction`
    /// for some existing [`crate::entity::Entity`].
    pub fn new<K: Into<EntityKey>>(entity_key: K, number_of_blocks: u64) -> Self {
        Self {
            entity_key: entity_key.into(),
            number_of_blocks,
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
