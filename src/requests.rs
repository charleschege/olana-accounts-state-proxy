use crate::RpcProxyResult;
use hyper::{Body, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A deserialized JSON request from a client
#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    /// The JSON version
    pub jsonrpc: String,
    /// The `id` field of valid JSON data
    pub id: u8,
    /// The RPC method to invoke
    pub method: String,
    /// The parameters to process
    pub params: (String, HashMap<String, String>),
}

impl RpcRequest {
    pub(crate) fn respond(&self, responder: &mut Response<Body>) -> RpcProxyResult<()> {
        let rpc_response = RpcResponse::<String>::new("Processing Valid data".to_owned());
        let ser_rpc_response = serde_json::to_string(&rpc_response)?;

        *responder.body_mut() = Body::from(ser_rpc_response);
        *responder.status_mut() = StatusCode::OK;

        Ok(())
    }

    /// Checks if the Rpc `Method` and `Encoding` are supported by the proxy server
    pub(crate) fn parameter_checks(&self, responder: &mut Response<Body>) -> RpcProxyResult<bool> {
        dbg!(&self);

        if !self.is_supported_method() {
            let mut error_data = String::new();
            error_data.push_str("Method `");
            error_data.push_str(&self.method);
            error_data.push_str("` Is Not Supported. Open a feature request issue on Github if you need this method to be supported");

            JsonError::new()
                .add_message("Method Not Supported")
                .add_data(&error_data)
                .response(responder)?;

            return Ok(false);
        }

        if let Some(encoding_data) = self.params.1.get("encoding") {
            let encoding: Encoding = encoding_data.as_str().into();

            dbg!(&encoding.is_supported(responder)?);

            if encoding.is_supported(responder)? {
            } else {
                return Ok(false);
            }
        } else {
            // Defaults to `Base64`
        }

        let parse_parameters = Parameter::parse(&self.params.1, responder)?;

        if !parse_parameters.0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// Check if a JSON Rpc Request Method is supported.
    fn is_supported_method(&self) -> bool {
        matches!(self.method.as_str(), "getAccountInfo")
    }
}

/// An RPC response ready to be serialized into JSON format
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    /// The JSON version used
    pub jsonrpc: String,
    /// The `id` field of valid JSON data
    pub id: u8,
    /// The result of the operation, which can return valid data or an error using [JsonError] struct
    pub result: T,
}

impl<T> RpcResponse<T> {
    /// Create a new [RpcResponse]. It takes a `T` as a parameter which is the data type
    /// to be returned to the user as valid JSON.
    ///
    /// #### Usage
    /// ```rust
    /// use solana_accounts_proxy::{RpcResponse};
    /// use serde::Serialize;
    ///
    /// #[derive(Debug, Serialize)]
    /// pub struct AccountMeta {
    ///     public_key: [u8; 32]
    /// }
    ///
    /// let data = AccountMeta { public_key: [0u8; 32] };
    /// let response = RpcResponse::<AccountMeta>::new(data);
    /// ```
    pub fn new(result: T) -> RpcResponse<T> {
        RpcResponse {
            jsonrpc: "2.0".to_owned(),
            id: 1,
            result,
        }
    }
}

/// A JSON version 2.0 error as specified
/// at [https://www.jsonrpc.org/specification#error_object](https://www.jsonrpc.org/specification#error_object)
#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct JsonError {
    /// The error code for the JSON data
    pub code: i16,
    /// The message to convey to the JSON client
    pub message: String,
    /// An optional String as arguments, this string can be used to convey extra data
    /// that can help a client process the error message accurately.
    pub data: Option<String>,
}

impl JsonError {
    /// Helper method to create a new JSON error.
    ///
    /// #### Usage
    /// ```rust
    /// use solana_accounts_proxy::JsonError;
    ///
    /// let mut json_error = JsonError::new();
    /// json_error.code = -32001;
    /// json_error.message = "Invalid field `public_key`";
    /// json_error.data = Some("The field `public_key` is not supported by the RPC server");
    ///
    /// ```
    pub fn new() -> Self {
        JsonError {
            code: -32000,
            message: String::default(),
            data: Option::None,
        }
    }

    /// Add a  JSON error code
    pub fn add_code(mut self, code: i16) -> Self {
        self.code = code;

        self
    }

    /// Add a  JSON error message
    pub fn add_message(mut self, message: &str) -> Self {
        self.message = message.to_owned();

        self
    }

    /// Add  JSON error data
    pub fn add_data(mut self, data: &str) -> Self {
        self.data = Some(data.to_owned());

        self
    }

    /// Add the error data to the `hyper::Response` body
    pub fn response(self, responder: &mut Response<Body>) -> RpcProxyResult<()> {
        let rpc_response = RpcResponse::<JsonError>::new(self);

        let ser_rpc_response = serde_json::to_string(&rpc_response)?;

        *responder.body_mut() = Body::from(ser_rpc_response);
        *responder.status_mut() = StatusCode::BAD_REQUEST;

        Ok(())
    }
}

impl Default for JsonError {
    fn default() -> Self {
        JsonError::new()
    }
}

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
    pub fn is_supported(&self, responder: &mut Response<Body>) -> RpcProxyResult<bool> {
        match self {
            Self::UnsupportedEncoding(encoding_value) => {
                let mut error_data = String::new();
                error_data.push_str("Encoding format `");
                error_data.push_str(encoding_value);
                error_data.push_str("` is not supported. ");
                error_data.push_str("Encoding formats available are, `Base64`, `Base64+zstd`, `Base58` and `JsonParsed`");

                JsonError::new()
                    .add_message("Unsupported Encoding Format")
                    .add_data(&error_data)
                    .response(responder)?;

                Ok(false)
            }
            _ => Ok(true),
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
    fn parse(
        parameters: &HashMap<String, String>,
        responder: &mut Response<Body>,
    ) -> RpcProxyResult<(bool, HashMap<String, Parameter>)> {
        let mut parameter: Parameter;
        let mut all_parameters = HashMap::<String, Parameter>::new();

        for (key, _) in parameters.iter() {
            parameter = key.as_str().into();

            if let Parameter::UnsupportedParameter(unsupported_value) = parameter {
                JsonError::new()
                    .add_message("Parameter Not Supported")
                    .add_data(&unsupported_value)
                    .response(responder)?;

                return Ok((false, all_parameters));
            }

            all_parameters.insert(key.clone(), parameter);
        }

        Ok((true, all_parameters))
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
