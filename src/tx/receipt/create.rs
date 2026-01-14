use alloy::primitives::{Address, U256};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{EntityKey, contract::ArkivAbi::ArkivEntityCreated};

/// Represents the result of creating an entity.
/// Contains the entity key and its expiration block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct CreateReceipt {
    /// The key of the entity.
    pub entity_key: EntityKey,
    /// The owner of the entity.
    pub owner_address: Address,
    /// The block number at which the entity expires.
    pub expiration_block: U256,
    /// The cost of the transaction in wei
    pub cost: U256,
}

impl From<ArkivEntityCreated> for CreateReceipt {
    fn from(data: ArkivEntityCreated) -> Self {
        Self {
            entity_key: data.entityKey.into(),
            owner_address: data.ownerAddress,
            expiration_block: data.expirationBlock,
            cost: data.cost,
        }
    }
}
