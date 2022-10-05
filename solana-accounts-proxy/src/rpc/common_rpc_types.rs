use serde::Serialize;
use serde_json::{Map, Value as JsonValue};

/// Slot context
#[derive(Debug, Serialize)]
pub struct Context {
    /// The period of time for which each leader ingests transactions and produces a block.
    pub slot: i64,
}

impl Context {
    /// Converts the [Context] into [serde_json::Value] and then inserts it to the
    /// `result` map
    pub fn as_json_value(&self, map: &mut Map<String, JsonValue>) {
        let mut slot = Map::new();
        slot.insert("slot".into(), self.slot.into());

        map.insert("context".into(), slot.into());
    }
}
