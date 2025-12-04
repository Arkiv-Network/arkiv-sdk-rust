use alloy::primitives::Address;
use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::EntityKey;

/// Represents the result of changing ownership of an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct ChownReceipt {
    /// The key of the entity that was transferred.
    pub entity_key: EntityKey,
    /// The previous owner of the entity.
    pub prev_owner: Address,
    /// The current owner of the entity.
    pub curr_owner: Address,
}
