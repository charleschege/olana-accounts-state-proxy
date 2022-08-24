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
}

impl Default for JsonError {
    fn default() -> Self {
        JsonError::new()
    }
}
