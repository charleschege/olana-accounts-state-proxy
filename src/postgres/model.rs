use core::fmt;
use tokio_postgres::Row;

/// Raw information from SQL query to getAccountInfo
pub struct GetAccountInfoRow {
    pubkey: String,
    slot: i64,
    write_version: i64,
    data: Vec<u8>,
    executable: bool,
    pub(crate) owner_id: i64,
    lamports: i64,
    rent_epoch: i64,
}

impl fmt::Debug for GetAccountInfoRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GetAccountInfoRow")
            .field("pubkey", &self.pubkey)
            .field("slot", &self.slot)
            .field("write_version", &self.write_version)
            .field("data", &hex::encode(&self.data))
            .field("executable", &self.executable)
            .field("owner_id", &self.owner_id)
            .field("lamports", &self.lamports)
            .field("rent_epoch", &self.rent_epoch)
            .finish()
    }
}

impl From<&Row> for GetAccountInfoRow {
    fn from(row: &Row) -> Self {
        let pubkey: String = row.get(0);
        let slot: i64 = row.get(1);
        let write_version: i64 = row.get(2);
        let data: Vec<u8> = row.get(3);
        let executable: bool = row.get(4);
        let owner_id: i64 = row.get(5);
        let lamports: i64 = row.get(6);
        let rent_epoch: i64 = row.get(7);

        GetAccountInfoRow {
            pubkey,
            slot,
            write_version,
            data,
            executable,
            owner_id,
            lamports,
            rent_epoch,
        }
    }
}

impl GetAccountInfoRow {
    /// Convert to JSON format
    pub fn to_json(&self, encoding: crate::Encoding, owner: &str) -> json::JsonValue {
        json::object! {
            "context": json::object! {
                "slot": self.slot,
            },
            "value": json::object! {
                "data": json::array![
                    encoding.encode(&self.data),
                    encoding.into_string(),
                ],
                "executable": self.executable,
                "lamports": self.lamports,
                "owner": owner,
                "rentEpoch": self.rent_epoch
            }
        }
    }
}

/*
{
    "jsonrpc": "2.0",
    "result": {
      "context": {
        "slot": 1
      },
      "value": {
        "data": [
          "11116bv5nS2h3y12kD1yUKeMZvGcKLSjQgX6BeV7u1FrjeJcKfsHRTPuR3oZ1EioKtYGiYxpxMG5vpbZLsbcBYBEmZZcMKaSoGx9JZeAuWf",
          "base58"
        ],
        "executable": false,
        "lamports": 1000000000,
        "owner": "11111111111111111111111111111111",
        "rentEpoch": 2
      }
    },
    "id": 1
  }
*/
