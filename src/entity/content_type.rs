pub mod error;
pub mod mime;

pub use error::Error;

/// `ContentType` in this context refers to what is now called `MediaType`
/// but is more commonly referred to as `MIME`.
///
/// Validation should adhere to RFC 2045 and RFC 7231 `MIME` type standards.
/// In addition, Arkiv `ContentType` validation requires the source length <= 128 characters.
///
/// Popular crates such as `mime` and `media_type` are supported, as well as
/// `hyper` or `reqwest` content type headers. For `MIME` validation without
/// the need of external crates, use [`crate::entity::content_type::mime::Mime`].
#[derive(Debug, Clone)]
pub struct ContentType<Mime: Into<String> + AsRef<str>>(pub(crate) Mime);

impl TryFrom<&str> for ContentType<String> {
    type Error = Error;
    fn try_from(source: &str) -> Result<Self, Self::Error> {
        ContentType(source.into()).validate()
    }
}
impl TryFrom<String> for ContentType<String> {
    type Error = Error;
    fn try_from(source: String) -> Result<Self, Self::Error> {
        ContentType(source).validate()
    }
}
impl TryFrom<ContentType<&str>> for ContentType<String> {
    type Error = Error;
    fn try_from(value: ContentType<&str>) -> Result<Self, Self::Error> {
        Ok(ContentType(value.0.into()))
    }
}
impl<Source: Into<String> + AsRef<str>> TryFrom<mime::Mime<Source>> for ContentType<Source> {
    type Error = Error;
    fn try_from(value: mime::Mime<Source>) -> Result<Self, Self::Error> {
        Ok(ContentType(value.0))
    }
}

impl<Mime: Into<String> + AsRef<str>> ContentType<Mime> {
    pub const fn new(mime: Mime) -> ContentType<Mime> {
        Self(mime)
    }

    /// A reference to the underlying source.
    pub fn source(&self) -> &str {
        self.0.as_ref()
    }

    /// Arkiv `ContentType` validation requires the source length <= 128 characters.
    const fn validate_source(source: &str) -> Result<(), Error> {
        if source.len() > 128 {
            return Err(Error::LengthExceeded);
        }
        Ok(())
    }

    /// Return an error if validation fails.
    fn validate(self) -> Result<Self, Error> {
        Self::validate_source(self.source())?;

        Ok(self)
    }
}

#[cfg(test)]
mod mime_tests {
    // TODO: Add test with content type from hypr/reqwest, add these examples to docs
    use super::{ContentType, Error, mime::Mime as ConstMime};

    #[test]
    fn control() {
        const CONTROL_CONTENT_TYPE: &str = r#"application/json; version="1";mode=debug"#;

        // compile time checks
        const _CONTROL_COMPILE_TIME: ConstMime<&str> = ConstMime::new(CONTROL_CONTENT_TYPE);

        // runtime checks
        assert!(ContentType::try_from(CONTROL_CONTENT_TYPE).is_ok());

        // parsed mime compatibility
        let mime: mime::Mime = "application/json".parse().unwrap();
        assert!(ContentType::try_from(mime.to_string()).is_ok());
    }

    #[test]
    fn length_exceeded() {
        const VALID_MIME_LENGTH_EXCEEDED: ConstMime<&str> = ConstMime::new(
            "application/vnd.example.super-long-custom-format+json;version=42;mode=fast;region=us-west-2;retry=5;debug=true;feature=experimental",
        );
        assert_eq!(
            ContentType::<&str>::validate_source(VALID_MIME_LENGTH_EXCEEDED.source()).err(),
            Some(Error::LengthExceeded)
        );
    }

    #[test]
    fn missing_type_subtype_separator() {
        assert_eq!(
            ConstMime::<&str>::validate_source("applicationjson;version=1").err(),
            Some(Error::MissingTypeSeparator)
        );
    }

    #[test]
    fn missing_type() {
        assert_eq!(
            ConstMime::<&str>::validate_source("/json;version=1").err(),
            Some(Error::MissingType)
        );
    }

    #[test]
    fn invalid_type_char() {
        assert_eq!(
            ConstMime::<&str>::validate_source("applic@tion/json;version=1").err(),
            Some(Error::InvalidTypeChar)
        );
    }

    #[test]
    fn missing_subtype() {
        assert_eq!(
            ConstMime::<&str>::validate_source("application/;version=1").err(),
            Some(Error::MissingSubtype)
        );
    }

    #[test]
    fn invalid_subtype_char() {
        assert_eq!(
            ConstMime::<&str>::validate_source("application/custom@json;version=1").err(),
            Some(Error::InvalidSubtypeChar)
        );
        assert_eq!(
            ConstMime::<&str>::validate_source("application/json ;version=1").err(),
            Some(Error::InvalidSubtypeChar)
        );
    }

    #[test]
    fn missing_parameter_separator() {
        assert_eq!(
            ConstMime::<&str>::validate_source("application/jsonversion=1").err(),
            Some(Error::MissingParameterSeparator)
        );
        assert_eq!(
            ConstMime::<&str>::validate_source("application/jsonversion=1").err(),
            Some(Error::MissingParameterSeparator)
        );
        assert_eq!(
            ConstMime::<&str>::validate_source("application/jsonversion=1;mode=debug").err(),
            Some(Error::MissingParameterSeparator)
        );
    }

    #[test]
    fn invalid_parameter_key() {
        assert_eq!(
            ConstMime::<&str>::validate_source("application/json;versi@n=1").err(),
            Some(Error::InvalidParameterKey)
        );
    }

    #[test]
    fn missing_parameter_assignment() {
        assert_eq!(
            ConstMime::<&str>::validate_source("application/json;version1").err(),
            Some(Error::MissingParameterAssignment)
        );
    }

    #[test]
    fn invalid_parameter_value() {
        assert_eq!(
            ConstMime::<&str>::validate_source("application/json;version=1@").err(),
            Some(Error::InvalidParameterValue)
        );
    }
}
