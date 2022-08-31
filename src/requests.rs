use std::collections::HashMap;

/// The supported Solana RPC encoding formats
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoding {
    /// The encoding of the response data is Base64 encoding format.
    Base64,
    /// The encoding of the response data is Base58 encoding format.
    /// `NOTE: Base58 is slower than Base64, always prefer Base64.`
    Base58,
    /// The encoding of the response data should use
    /// internal parsers to return JSON encoded response data.
    JsonParsed,
    /// The encoding of the  response data is Base64 encoding format compressed
    /// using `zstd` compression algorithm.
    Base64Zstd,
    /// The encoding provided is not supported by the proxy server. Try Base64 encoding
    UnsupportedEncoding(String),
}

impl Encoding {
    /// Check if the encoding from the RPC request is supported by the proxy server
    pub fn is_supported(&self) -> Result<(), String> {
        match self {
            Self::UnsupportedEncoding(encoding_value) => {
                let mut error_data = String::new();
                error_data.push_str("Encoding format `");
                error_data.push_str(encoding_value);
                error_data.push_str("` is not supported. ");
                error_data.push_str("Encoding formats available are, `Base64`, `Base64+zstd`, `Base58` and `JsonParsed`");

                Err(error_data)
            }
            _ => Ok(()),
        }
    }
}

impl From<&str> for Encoding {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "base64" => Encoding::Base64,
            "base58" => Encoding::Base58,
            "jsonparsed" => Encoding::JsonParsed,
            "base64+zstd" => Encoding::Base64Zstd,
            _ => Encoding::UnsupportedEncoding(value.to_owned()),
        }
    }
}

/// The parameters that are used to filter and return the required data.
/// Some popular parameters are `encoding` and `commitment`
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Parameter {
    /// The encoding to use when returning JSON data as specified by [Encoding] enum.
    Encoding,
    /// The current state of a block or transaction as specified by [Encoding] enum.
    Commitment,
    /// limit the returned account data using the provided offset: <usize> and length: <usize> fields;
    /// only available for "base58", "base64" or "base64+zstd" encodings.
    DataSlice,
    /// sets the minimum slot that the request can be evaluated at.
    MinContextSlot,
    /// The parameter provided is not supported.
    UnsupportedParameter(String),
}

impl Parameter {
    /// Check if parameters provided are supported
    pub fn parse(
        parameters: &HashMap<String, String>,
    ) -> Result<HashMap<String, Parameter>, String> {
        let mut parameter: Parameter;
        let mut all_parameters = HashMap::<String, Parameter>::new();

        for (key, _) in parameters.iter() {
            parameter = key.as_str().into();

            if let Parameter::UnsupportedParameter(unsupported_value) = parameter {
                let mut error = String::new();
                error.push_str("Parameter `");
                error.push_str(&unsupported_value);
                error.push_str("` Not Supported.");

                return Err(error);
            }

            all_parameters.insert(key.clone(), parameter);
        }

        Ok(all_parameters)
    }
}

impl From<&str> for Parameter {
    fn from(value: &str) -> Self {
        match value {
            "encoding" => Parameter::Encoding,
            "commitment" => Parameter::Commitment,
            "dataSlice" => Parameter::DataSlice,
            "minContextSlot" => Parameter::MinContextSlot,
            _ => Parameter::UnsupportedParameter(value.to_owned()),
        }
    }
}
