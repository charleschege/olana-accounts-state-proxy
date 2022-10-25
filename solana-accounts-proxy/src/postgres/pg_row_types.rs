use crate::{Account, AccountInfo, Context, Encoding};
use jsonrpsee::core::RpcResult;
use serde_json::Value as SerdeJsonValue;
use tokio_postgres::Row;

/// Enables easier serialization from a postgres `Row` from the `getAccountInfo` query
#[derive(Debug)]
pub struct GetAccountInfoRow {
    pub(crate) context: Context,
    pub(crate) value: Account,
}

impl From<Row> for GetAccountInfoRow {
    fn from(row: Row) -> Self {
        let slot: i64 = row.get(0);
        let slot = slot as u64;
        let data: Vec<u8> = row.get(1);
        let executable: bool = row.get(2);
        let owner: String = row.get(3);
        let lamports: i64 = row.get(4);
        let rent_epoch: i64 = row.get(5);

        GetAccountInfoRow {
            context: Context {
                slot,
                api_version: Option::None,
            },
            value: Account {
                data,
                executable,
                owner,
                lamports,
                rent_epoch,
            },
        }
    }
}

/// Enables easier serialization from a postgres `Row` from the `getAccountInfo` query
#[derive(Debug)]
pub struct GetProgramAccountsRow;

impl GetProgramAccountsRow {
    /// Convert a postgres Row into [AccountInfo] then to JSON format in one method.
    pub fn from_row(rows: Vec<Row>, encoding: Encoding) -> RpcResult<Vec<SerdeJsonValue>> {
        let mut account_info_list = Vec::<SerdeJsonValue>::new();

        for row in rows {
            let pubkey: String = row.get(0);
            let owner: String = row.get(1);
            let lamports: i64 = row.get(2);
            let executable: bool = row.get(3);
            let rent_epoch: i64 = row.get(4);
            let data: Vec<u8> = row.get(5);

            let account = Account {
                data,
                executable,
                owner,
                lamports,
                rent_epoch,
            };

            let account_info = AccountInfo { pubkey, account };
            let to_json = account_info.as_json_value(encoding)?;

            account_info_list.push(to_json);
        }

        Ok(account_info_list)
    }
}
