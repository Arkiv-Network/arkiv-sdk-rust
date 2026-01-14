use alloy::primitives::{Address, U256};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{EntityKey, contract::ArkivAbi::ArkivEntityBTLExtended};

/// A receipt returned by the network from a [`crate::tx::Transaction`] containing an [`crate::tx::ops::Extend`] operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct ExtendReceipt {
    /// The key of the entity.
    pub entity_key: EntityKey,
    /// The owner of the entity.
    pub owner_address: Address,
    /// The old expiration block of the entity.
    pub old_expiration_block: U256,
    /// The new expiration block of the entity.
    pub new_expiration_block: U256,
    /// The cost of the transaction in wei.
    pub cost: U256,
}
impl From<ArkivEntityBTLExtended> for ExtendReceipt {
    fn from(data: ArkivEntityBTLExtended) -> Self {
        Self {
            entity_key: data.entityKey.into(),
            owner_address: data.ownerAddress,
            old_expiration_block: data.oldExpirationBlock,
            new_expiration_block: data.newExpirationBlock,
            cost: data.cost,
        }
    }
}
