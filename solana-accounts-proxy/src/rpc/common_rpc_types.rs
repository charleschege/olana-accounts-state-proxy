use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as JsonValue};

/// Slot context
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    /// The period of time for which each leader ingests transactions and produces a block.
    pub slot: u64,
    /// The version of the api
    pub api_version: Option<String>,
}

impl Context {
    /// Converts the [Context] into [serde_json::Value] and then inserts it to the
    /// `result` map
    pub fn as_json_value(&self, map: &mut Map<String, JsonValue>) {
        let mut slot = Map::new();
        slot.insert("slot".into(), self.slot.into());

        if let Some(api_version) = self.api_version.as_ref() {
            map.insert("apiVersion".into(), api_version.as_str().into());
        }
    }
}

/// The result of a n RPC request
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RpcResult<T> {
    jsonrpc: String,
    id: u8,
    result: T,
}

impl<T> RpcResult<T> where T: serde::de::DeserializeOwned + std::fmt::Debug {}

/// The value of the data contained in the RPC request
#[derive(Debug, Deserialize, Serialize)]
pub struct RpcResultData<U> {
    context: Context,
    value: U,
}

impl<U> RpcResult<U> where U: serde::de::DeserializeOwned + std::fmt::Debug {}
