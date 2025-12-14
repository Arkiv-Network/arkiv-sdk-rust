#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    #[error("`ContentType` exceeds maximum length of `[char; 128]`.")]
    LengthExceeded,

    #[error("`ContentType` missing type-subtype separator: '/'.")]
    MissingTypeSeparator,

    #[error("`ContentType` missing type before type-subtype separator: '/'.")]
    MissingType,

    #[error("`ContentType` type contains an invalid character")]
    InvalidTypeChar,

    #[error("`ContentType` missing subtype after type-subtype separator: '/'.")]
    MissingSubtype,

    #[error("`ContentType` subtype contains an invalid character")]
    InvalidSubtypeChar,

    #[error("`ContentType` missing parameter separator, expected ';'.")]
    MissingParameterSeparator,

    #[error("`ContentType` invalid character in parameter key.")]
    InvalidParameterKey,

    #[error("`ContentType` invalid parameter key or missing '='.")]
    MissingParameterAssignment,

    #[error("`ContentType` invalid character or empty parameter value.")]
    InvalidParameterValue,
}
impl Error {
    pub(super) const fn as_static_str(&self) -> &'static str {
        match self {
            Self::LengthExceeded => "`ContentType` exceeds maximum length of `[char; 128]`.",
            Self::MissingTypeSeparator => "`ContentType` missing type-subtype separator: '/'.",
            Self::MissingType => "`ContentType` missing type before type-subtype separator: '/'.",
            Self::InvalidTypeChar => "`ContentType` type contains an invalid character",
            Self::MissingSubtype => {
                "`ContentType` missing subtype after type-subtype separator: '/'."
            }
            Self::InvalidSubtypeChar => "`ContentType` subtype contains an invalid character",
            Self::MissingParameterSeparator => {
                "`ContentType` invalid parameter separator, expected ';'."
            }
            Self::InvalidParameterKey => "`ContentType` invalid character in parameter key.",
            Self::MissingParameterAssignment => {
                "`ContentType` invalid parameter key or missing '='."
            }
            Self::InvalidParameterValue => {
                "`ContentType` invalid character or empty parameter value."
            }
        }
    }
}
