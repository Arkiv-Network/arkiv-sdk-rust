use alloy::primitives::{Address, U256};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{EntityKey, contract::ArkivAbi::ArkivEntityUpdated};

/// Data returned by the network after updating an [`crate::entity::Entity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct UpdateReceipt {
    /// The key of the entity.
    pub entity_key: EntityKey,
    /// The owner of the entity.
    pub owner_address: Address,
    /// The old block number at which the entity would have expired.
    pub old_expiration_block: U256,
    /// The new block number at which the entity expires.
    pub new_expiration_block: U256,
    /// The cost of the transaction in wei.
    pub cost: U256,
}
impl From<ArkivEntityUpdated> for UpdateReceipt {
    fn from(data: ArkivEntityUpdated) -> Self {
        Self {
            entity_key: data.entityKey.into(),
            owner_address: data.ownerAddress,
            old_expiration_block: data.oldExpirationBlock,
            new_expiration_block: data.newExpirationBlock,
            cost: data.cost,
        }
    }
}
