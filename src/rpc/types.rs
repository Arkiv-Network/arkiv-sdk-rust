// TODO: Add docs
//! Types used by the arkiv storage network for interacting with entities via RPC.

use serde::{Deserialize, Serialize};

mod event;
#[cfg(feature = "pubsub")]
pub use event::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryOpts {
    /// Historical block number for query
    pub at_block: Option<u64>,
    /// Which fields to return
    pub include_data: Option<IncludeData>,
    /// Ordering by attributes
    pub order_by: Option<Vec<OrderByAttribute>>,
    /// Pagination limit
    pub results_per_page: Option<u64>,
    /// Pagination cursor from previous response
    pub cursor: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IncludeData {
    /// Entity key hash
    pub key: bool,
    /// User-defined attributes
    pub attributes: bool,
    /// System-generated attributes ($sequence, $creator)
    pub synthetic_attributes: bool,
    /// Entity data content
    pub payload: bool,
    /// MIME type
    pub content_type: bool,
    /// Expiration block number
    pub expiration: bool,
    /// Owner address
    pub owner: bool,
    /// Creation block number
    pub created_at_block: bool,
    /// Last modification block
    pub last_modified_at_block: bool,
    /// Transaction index
    pub transaction_index_in_block: bool,
    /// Operation index
    pub operation_index_in_transaction: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OrderByAttribute {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "desc")]
    pub descending: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockTiming {
    pub current_block: u64,
    pub current_block_time: u64,
    #[serde(rename = "duration")]
    pub block_duration: u64,
}
