use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::entity::EntityKey;

/// Type representing a delete transaction in GolemBase.
#[derive(Debug, Clone, Copy, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct Delete(EntityKey);
impl Delete {
    pub fn new(entity_key: EntityKey) -> Self {
        Self(entity_key)
    }
    pub fn entity_key(&self) -> EntityKey {
        self.0
    }
}
impl From<EntityKey> for Delete {
    fn from(value: EntityKey) -> Self {
        Self(value)
    }
}
