use tokio_postgres::Row;

use crate::{AccountInfo, Context};

/// Enables easier serialization from a postgres `Row` from the `getAccountInfo` query
#[derive(Debug)]
pub struct GetAccountInfoRow {
    pub(crate) context: Context,
    pub(crate) value: AccountInfo,
}

impl From<&Row> for GetAccountInfoRow {
    fn from(row: &Row) -> Self {
        let slot: i64 = row.get(0);
        let data: Vec<u8> = row.get(1);
        let executable: bool = row.get(2);
        let owner: String = row.get(3);
        let lamports: i64 = row.get(4);
        let rent_epoch: i64 = row.get(5);

        GetAccountInfoRow {
            context: Context { slot },
            value: AccountInfo {
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
pub struct GetProgramAccountsRow {
    context: Context,
    pubkey: String,
    value: AccountInfo,
}
/*
impl From<&Row> for GetProgramAccountsRow {
    fn from(row: &Row) -> Self {
        let slot: i64 = row.get(0);

        GetProgramAccountsRow {}
    }
}*/
