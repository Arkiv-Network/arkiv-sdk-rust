use alloy_rlp::{Decodable, Encodable};
use serde::{Deserialize, Serialize};

use crate::entity::EntityKey;

/// Type representing a delete transaction in GolemBase.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Delete(EntityKey);
impl Delete {
    pub fn new(entity_key: EntityKey) -> Self {
        Self(entity_key)
    }
    pub fn entity_key(&self) -> EntityKey {
        self.0
    }
}
impl Encodable for Delete {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        self.0.encode(out)
    }
}
impl Decodable for Delete {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        EntityKey::decode(buf).map(Self)
    }
}
impl From<EntityKey> for Delete {
    fn from(value: EntityKey) -> Self {
        Self(value)
    }
}
