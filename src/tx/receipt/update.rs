use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{EntityKey, contract::ArkivAbi::EntityUpdated};

/// Data returned by the network after updating an [`crate::entity::Entity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct UpdateReceipt {
    /// The key of the entity.
    pub entity_key: EntityKey,
    /// The block number at which the entity expires.
    pub expiration_block: u64,
}
impl From<EntityUpdated> for UpdateReceipt {
    fn from(data: EntityUpdated) -> Self {
        Self {
            entity_key: data.entityKey.into(),
            expiration_block: data.expirationBlock.try_into().unwrap_or_default(),
        }
    }
}
