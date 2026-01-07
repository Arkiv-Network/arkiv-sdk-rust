pub mod types;

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
