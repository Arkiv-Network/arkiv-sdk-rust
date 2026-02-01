use alloy::primitives::Address;
use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{EntityKey, contract::ArkivAbi::ArkivEntityDeleted};

/// Data returned by the network after deleting an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct DeleteReceipt {
    /// The key of the entity that was deleted.
    pub entity_key: EntityKey,
    /// The owner of the entity that was deleted.
    pub owner_address: Address,
}
impl From<ArkivEntityDeleted> for DeleteReceipt {
    fn from(data: ArkivEntityDeleted) -> Self {
        Self {
            entity_key: data.entityKey.into(),
            owner_address: data.ownerAddress,
        }
    }
}
