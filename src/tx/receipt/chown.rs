use alloy::primitives::Address;
use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{EntityKey, contract::ArkivAbi::ArkivEntityOwnerChanged};

/// Represents the result of changing ownership of an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct ChownReceipt {
    /// The key of the entity that was transferred.
    pub entity_key: EntityKey,
    /// The previous owner of the entity.
    pub old_owner_address: Address,
    /// The new owner of the entity.
    pub new_owner_address: Address,
}
impl From<ArkivEntityOwnerChanged> for ChownReceipt {
    fn from(data: ArkivEntityOwnerChanged) -> Self {
        Self {
            entity_key: data.entityKey.into(),
            old_owner_address: data.oldOwnerAddress,
            new_owner_address: data.newOwnerAddress,
        }
    }
}
