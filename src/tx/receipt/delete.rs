use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{EntityKey, contract::ArkivAbi::EntityDeleted};

/// Data returned by the network after deleting an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct DeleteReceipt {
    /// The key of the entity that was deleted.
    pub entity_key: EntityKey,
}
impl From<EntityDeleted> for DeleteReceipt {
    fn from(data: EntityDeleted) -> Self {
        Self {
            entity_key: data.entityKey.into(),
        }
    }
}
