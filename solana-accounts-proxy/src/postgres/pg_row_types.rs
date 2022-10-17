use tokio_postgres::Row;

use crate::{Account, Context};

/// Enables easier serialization from a postgres `Row` from the `getAccountInfo` query
#[derive(Debug)]
pub struct GetAccountInfoRow {
    pub(crate) context: Context,
    pub(crate) value: Account,
}

impl From<&Row> for GetAccountInfoRow {
    fn from(row: &Row) -> Self {
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
pub struct GetProgramAccountsRow {
    pub(crate) values: Vec<Account>,
}

impl GetProgramAccountsRow {
    fn from_row(rows: Vec<Row>) -> Vec<Account> {
        let mut accounts = Vec::<Account>::new();

        for row in rows {
            let slot: i64 = row.get(0);
            let slot = slot as u64;
            let data: Vec<u8> = row.get(1);
            let executable: bool = row.get(2);
            let owner: String = row.get(3);
            let lamports: i64 = row.get(4);
            let rent_epoch: i64 = row.get(5);

            let account = Account {
                data,
                executable,
                owner,
                lamports,
                rent_epoch,
            };

            accounts.push(account);
        }

        accounts
    }
}
