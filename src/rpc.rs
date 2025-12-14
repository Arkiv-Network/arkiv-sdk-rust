use serde::{Deserialize, Serialize};

/// Available RPC methods for interacting with the Arkiv network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArkivRpcMethod {
    /// `arkiv_query`
    ///
    /// Filter and retrieve entities matching a query expression.
    Query,
    /// `arkiv_getEntityCount`
    ///
    /// Returns total number of entities at current block.
    GetEntityCount,
    /// `arkiv_getNumberOfUsedSlots`
    ///
    /// Returns storage slot usage accounting.
    GetNumberOfUsedSlots,
    /// `arkiv_getBlockTiming`
    ///
    /// Returns current block timing information.
    GetBlockTiming,
}
impl ArkivRpcMethod {
    const ARKIV_QUERY: &str = "arkiv_query";
    const ARKIV_GET_ENTITY_COUNT: &str = "arkiv_getEntityCount";
    const ARKIV_GET_NUMBER_OF_USED_SLOTS: &str = "arkiv_getNumberOfUsedSlots";
    const ARKIV_GET_BLOCK_TIMING: &str = "arkiv_getBlockTiming";

    /// Return the corresponding method as a `&'static str`.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Query => Self::ARKIV_QUERY,
            Self::GetEntityCount => Self::ARKIV_GET_ENTITY_COUNT,
            Self::GetNumberOfUsedSlots => Self::ARKIV_GET_NUMBER_OF_USED_SLOTS,
            Self::GetBlockTiming => Self::ARKIV_GET_BLOCK_TIMING,
        }
    }
}
impl std::ops::Deref for ArkivRpcMethod {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
impl From<ArkivRpcMethod> for std::borrow::Cow<'static, str> {
    fn from(value: ArkivRpcMethod) -> Self {
        value.as_str().into()
    }
}

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
