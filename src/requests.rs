use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u8,
    pub method: String,
    pub params: (String, HashMap<String, String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: u8,
    pub result: T,
}

impl<T> RpcResponse<T> {
    pub fn new(result: T) -> RpcResponse<T> {
        RpcResponse {
            jsonrpc: "2.0".to_owned(),
            id: 1,
            result,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct JsonError {
    pub code: i16,
    pub message: String,
    pub data: Option<String>,
}

impl JsonError {
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
