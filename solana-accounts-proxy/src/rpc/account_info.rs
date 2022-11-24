use core::fmt;
use jsonrpsee::core::RpcResult;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as SerdeJsonValue};
use tokio::time::Instant;

/// AccountInfo which is just an [Account] with an additional field of `pubkey`
/// Account information
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    /// The public key of the account
    pub pubkey: String,
    /// An [Account]
    pub account: Account,
}

impl AccountInfo {
    /// Convert the `AccountInfo` into a JSON value to pass to the
    /// RPC response
    pub fn as_json_value(&self, encoding: crate::Encoding) -> RpcResult<SerdeJsonValue> {
        let timer = Instant::now();

        let mut map = Map::new();
        map.insert("pubkey".to_owned(), self.pubkey.as_str().into());

        self.account.as_json_value(encoding, &mut map)?;

        let outcome = Instant::now().duration_since(timer);

        tracing::trace!(
            "TIME TO ENCODE TO `AccountInfo` JSON: {}s {}ms {}ns",
            outcome.as_secs(),
            outcome.subsec_millis(),
            outcome.subsec_nanos()
        );

        Ok(map.into())
    }
}

/// An Account
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
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

impl Account {
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
        map: &mut Map<String, SerdeJsonValue>,
    ) -> RpcResult<()> {
        let mut json_result = Map::new();
        json_result.insert(
            "data".into(),
            SerdeJsonValue::Array(vec![
                encoding.encode(&self.data)?.into(),
                encoding.to_str().into(),
            ]),
        );
        json_result.insert("executable".into(), self.executable.into());
        json_result.insert("lamports".into(), self.lamports.into());
        json_result.insert("owner".into(), self.owner.clone().into());
        json_result.insert("rentEpoch".into(), self.rent_epoch.into());

        map.insert("account".into(), json_result.into());

        Ok(())
    }
}

impl fmt::Debug for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Account")
            .field("owner", &self.owner)
            .field("lamports", &self.lamports)
            .field("executable", &self.executable)
            .field("rent_epoch", &self.rent_epoch)
            .field("data", &hex::encode(&self.data))
            .finish()
    }
}
