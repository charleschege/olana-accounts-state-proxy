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
    pub fn as_json_value(&self) -> Map<String, JsonValue> {
        let mut context = Map::new();
        context.insert("slot".into(), self.slot.into());

        if let Some(api_version) = self.api_version.as_ref() {
            context.insert("apiVersion".into(), api_version.as_str().into());
        }

        context
    }

    /// Converts the [Context] into [serde_json::Value] and then inserts it to the
    /// `result` map
    pub fn insert_json_value(&self, map: &mut Map<String, JsonValue>) {
        let mut slot = Map::new();
        slot.insert("slot".into(), self.slot.into());

        if let Some(api_version) = self.api_version.as_ref() {
            slot.insert("apiVersion".into(), api_version.as_str().into());
        }

        map.insert("context".into(), slot.into());
    }
}

impl From<tokio_postgres::Row> for Context {
    fn from(row: tokio_postgres::Row) -> Self {
        let max: i64 = row.get(0);

        Context {
            slot: max as u64,
            api_version: Option::None, //TODO Add the API version here
        }
    }
}

/// The result of an RPC request
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RpcResult<T> {
    /// The JSON version
    pub jsonrpc: String,
    /// The ID
    pub id: u8,
    /// The result of the response
    pub result: T,
}

impl<T> RpcResult<T> where T: serde::de::DeserializeOwned + std::fmt::Debug {}

/// The value of the data contained in the RPC request
#[derive(Debug, Deserialize, Serialize)]
pub struct WithContext<U> {
    context: Context,
    value: U,
}

impl<U> WithContext<U>
where
    U: serde::de::DeserializeOwned + std::fmt::Debug + Default,
{
    /// Instantiate a new `WithContext` struct
    pub fn new(context: Context) -> WithContext<U> {
        WithContext {
            context,
            value: U::default(),
        }
    }

    /// Convert `WithContext` to JSON value
    pub fn as_json_value(self, value: JsonValue) -> Map<String, JsonValue> {
        let context_as_json = self.context.as_json_value();

        let mut map = Map::new();

        map.insert("context".into(), context_as_json.into());
        map.insert("value".into(), value);

        map
    }
}
