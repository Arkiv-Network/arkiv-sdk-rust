use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{EntityKey, eth::ArkivAbi::EntityExtended};

/// A receipt returned by the network from a [`crate::tx::Transaction`] containing an [`crate::tx::ops::Extend`] operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct ExtendReceipt {
    /// The key of the entity.
    pub entity_key: EntityKey,
    /// The old expiration block of the entity.
    pub old_expiration_block: u64,
    /// The new expiration block of the entity.
    pub new_expiration_block: u64,
}
impl From<EntityExtended> for ExtendReceipt {
    fn from(data: EntityExtended) -> Self {
        Self {
            entity_key: data.entityKey.into(),
            old_expiration_block: data.oldExpirationBlock.try_into().unwrap_or_default(),
            new_expiration_block: data.newExpirationBlock.try_into().unwrap_or_default(),
        }
    }
}
