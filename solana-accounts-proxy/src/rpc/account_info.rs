use core::fmt;
use jsonrpsee::core::RpcResult;
use serde::Serialize;
use serde_json::{Map, Value as JsonValue};

/// Account information
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    /// The data specific to the account
    pub data: Vec<u8>,
    /// Is the account executable
    pub executable: bool,
    /// Number of lamports held by the account
    pub lamports: i64,
    /// The owner of the account
    pub owner: String,
    /// Next epoch when rent will be collected
    pub rent_epoch: i64,
}

impl AccountInfo {
    /// (optional) dataSlice: <object> -
    /// limit the returned account data using the provided offset: <usize> and length: <usize> fields;
    /// only available for "base58", "base64" or "base64+zstd" encodings.
    pub fn as_data_slice(&mut self, offset: usize, length: usize) -> &mut Self {
        if offset == 0 && length == 0 {
            return self;
        }

        if length == 0 {
            let partial_data = self.data[offset..].to_vec();

            self.data = partial_data;

            self
        } else {
            let partial_data = self.data[offset..=length].to_vec(); //TODO test for accuracy if the range is inclusive or not

            self.data = partial_data;

            self
        }
    }

    /// Convert to JSON format
    pub fn as_json_value(
        &self,
        encoding: crate::Encoding,
        map: &mut Map<String, JsonValue>,
    ) -> RpcResult<()> {
        let mut json_result = Map::new();
        json_result.insert(
            "data".into(),
            JsonValue::Array(vec![
                encoding.encode(&self.data)?.into(),
                encoding.to_str().into(),
            ]),
        );
        json_result.insert("executable".into(), self.executable.into());
        json_result.insert("lamports".into(), self.lamports.into());
        json_result.insert("owner".into(), self.owner.clone().into());
        json_result.insert("rentEpoch".into(), self.rent_epoch.into());

        map.insert("value".to_owned(), json_result.into());

        Ok(())
    }
}

impl fmt::Debug for AccountInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountInfo")
            .field("owner", &self.owner)
            .field("lamports", &self.lamports)
            .field("executable", &self.executable)
            .field("rent_epoch", &self.rent_epoch)
            .field("data", &hex::encode(&self.data))
            .finish()
    }
}
