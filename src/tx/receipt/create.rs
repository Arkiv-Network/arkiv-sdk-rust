use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{EntityKey, contract::ArkivAbi::ArkivEntityCreated};

/// Represents the result of creating an entity.
/// Contains the entity key and its expiration block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct CreateReceipt {
    /// The key of the entity.
    pub entity_key: EntityKey,
    /// The block number at which the entity expires.
    pub expiration_block: u64,
}

impl From<ArkivEntityCreated> for CreateReceipt {
    fn from(data: ArkivEntityCreated) -> Self {
        Self {
            entity_key: data.entityKey.into(),
            expiration_block: data.expirationBlock.try_into().unwrap_or_default(),
        }
    }
}
