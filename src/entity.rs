//! Module for Arkiv entities and data types.
//! Defines core types such as annotations, hashes, and entity representations.

use alloy::primitives::{Address, U256};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

pub mod attribute;
pub mod btl;
pub mod content_type;

pub use attribute::{Attribute, NumericAttribute, StringAttribute};
pub use btl::BlocksToLive;
pub use content_type::ContentType;

/// A type alias for the hash used to identify entities in Arkiv.
pub type EntityKey = alloy::primitives::B256;

/// Represents an entity with data, BTL, and annotations.
/// Used for reading entity state from the chain.
#[derive(Debug, Clone, Default, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct Entity {
    pub key: EntityKey,
    pub value: Vec<u8>,
    pub content_type: String,
    pub expires_at: U256,
    pub owner: Address,
    pub created_at_block: U256,
    /// String attributes for the entity.
    pub string_attributes: Vec<StringAttribute>,
    /// Numeric attributes for the entity.
    pub numeric_attributes: Vec<NumericAttribute>,
}
