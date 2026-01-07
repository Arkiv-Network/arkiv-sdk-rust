// TODO: Add docs
//! Types used by the arkiv storage network for interacting with entities via RPC.

use serde::{Deserialize, Serialize};

mod event;
#[cfg(feature = "pubsub")]
pub use event::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryOpts {
    /// Historical block number for query
    at_block: Option<u64>,
    /// Which fields to return
    include_data: Option<IncludeData>,
    /// Ordering by attributes
    order_by: Option<Vec<OrderByAttribute>>,
    /// Pagination limit
    results_per_page: Option<u64>,
    /// Pagination cursor from previous response
    cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IncludeData {
    /// Entity key hash
    key: bool,
    /// User-defined attributes
    attributes: bool,
    /// System-generated attributes ($sequence, $creator)
    synthetic_attributes: bool,
    /// Entity data content
    payload: bool,
    /// MIME type
    content_type: bool,
    /// Expiration block number
    expiration: bool,
    /// Owner address
    owner: bool,
    /// Creation block number
    created_at_block: bool,
    /// Last modification block
    last_modified_at_block: bool,
    /// Transaction index
    transaction_index_in_block: bool,
    /// Operation index
    operation_index_in_transaction: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OrderByAttribute {
    name: String,
    #[serde(rename = "type")]
    r#type: String,
    #[serde(rename = "desc")]
    descending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockTiming {
    current_block: u64,
    current_block_time: u64,
    #[serde(rename = "duration")]
    block_duration: u64,
}
