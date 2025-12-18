use crate::entity::content_type::Error;

/// A minimal compile time checked, `const` friendly MIME string.
#[derive(Debug, Clone)]
pub struct Mime<Source: Into<String> + AsRef<str>>(pub(crate) Source);
// TODO: Provide links to the RFCs
impl Mime<&'static str> {
    /// Used for compile time validation of MIME types as `const` values.
    ///
    /// # Panics
    ///
    /// Panics if the source is not a valid `MIME` according to RFC 2045 and 7231.
    ///
    /// # Example
    ///
    /// ```rs,ignore
    /// use arkiv_sdk::entity::content_type::mime::Mime;
    ///
    /// pub const COMPILE_TIME_VALIDATED_MIME: Mime<&str> = Mime::new("application/vnd.example.long-format+json;version=42;mode=fast;debug=true;region=us-west-2;retry=5");
    /// ```
    pub const fn new(source: &'static str) -> Mime<&'static str> {
        match Self::validate_source(source) {
            Ok(()) => Mime(source),
            Err(err) => panic!("{}", err.as_static_str()),
        }
    }
}
impl<Source: Into<String> + AsRef<str>> Mime<Source> {
    /// A reference to the underlying source.
    pub fn source(&self) -> &str {
        self.0.as_ref()
    }

    /// Validate source according to RFC 2045 and RFC 7231 `MIME` type standards.
    pub(super) const fn validate_source(source: &str) -> Result<(), Error> {
        let bytes = source.as_bytes();

        // We must first find the type-subtype separator prior to validating the type
        let mut type_sep_index = 0;
        let mut contains_invalid_char = false;
        while type_sep_index < bytes.len() && bytes[type_sep_index] != b'/' {
            if !Self::is_type_token(bytes[type_sep_index]) {
                contains_invalid_char = true;
            }
            type_sep_index += 1;
        }
        if type_sep_index == bytes.len() {
            return Err(Error::MissingTypeSeparator);
        }
        if type_sep_index == 0 {
            return Err(Error::MissingType);
        }
        if contains_invalid_char {
            return Err(Error::InvalidTypeChar);
        }

        // Parse the subtype
        let mut subtype_start = type_sep_index + 1;
        let mut err = Ok(());
        while subtype_start < bytes.len() && bytes[subtype_start] != b';' {
            let byte = bytes[subtype_start];
            if !Self::is_subtype_token(byte) && err.is_ok() {
                if byte == b'=' {
                    // More than likely this is a typo or missing semicolon
                    err = Err(Error::MissingParameterSeparator);
                } else {
                    err = Err(Error::InvalidSubtypeChar);
                }
            }
            subtype_start += 1;
        }
        if subtype_start == type_sep_index + 1 {
            return Err(Error::MissingSubtype);
        }
        if err.is_err() {
            return err;
        }

        // Validate params
        let mut param_index = subtype_start;
        while param_index < bytes.len() {
            // Find param separator
            if bytes[param_index] != b';' {
                return Err(Error::MissingParameterSeparator);
            }
            param_index += 1;
            while bytes[param_index] == b' ' {
                param_index += 1;
            }

            // Parse param key
            let key_start = param_index;
            while param_index < bytes.len() && bytes[param_index] != b'=' {
                if !Self::is_parameter_token(bytes[param_index]) {
                    return Err(Error::InvalidParameterKey);
                }
                param_index += 1;
            }

            if param_index == key_start || param_index >= bytes.len() {
                return Err(Error::MissingParameterAssignment);
            }

            param_index += 1; // Skip '='

            let mut quoted_value_pair = 0;
            if bytes[param_index] == b'"' {
                quoted_value_pair = 1;
                param_index += 1;
            }

            // Parse param value
            let val_start = param_index;
            while param_index < bytes.len() && bytes[param_index] != b';' {
                let byte = bytes[param_index];
                if byte == b'"' {
                    quoted_value_pair += 1;
                }
                if !Self::is_parameter_token(byte) && byte != b'"'
                    || (byte == b'"' && quoted_value_pair > 2)
                {
                    return Err(Error::InvalidParameterValue);
                }
                param_index += 1;
            }

            if param_index == val_start {
                return Err(Error::InvalidParameterValue);
            }
        }

        Ok(())
    }

    const fn is_type_token(byte: u8) -> bool {
        matches!(byte,
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'!' | b'#' | b'$' | b'&' |
            b'-' | b'^' | b'_' | b'.' | b'+'
        )
    }

    const fn is_subtype_token(byte: u8) -> bool {
        Self::is_type_token(byte)
    }

    const fn is_parameter_token(byte: u8) -> bool {
        matches!(byte,
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' |
            b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' |
            b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
        )
    }
}
