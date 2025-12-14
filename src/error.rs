pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum Error {
    #[error(transparent)]
    ContentType(#[from] crate::entity::content_type::Error),

    #[error("failed to convert string to MIME: {0}")]
    MimeFromStr(String),

    #[error("Missing EntityKey")]
    MissingEntityKey,

    #[error("Missing BlocksToLive")]
    MissingBtl,

    #[error("Missing ContentType")]
    MissingContentType,

    #[error("Missing Payload")]
    MissingPayload,

    #[error(transparent)]
    ContractError(#[from] crate::contract::Error),
}
