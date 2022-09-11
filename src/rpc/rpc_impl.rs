use crate::{
    Commitment, Encoding, GetAccountInfoRow, Parameters, PgConnection, PubKey, RpcProxyServer,
    CLIENT,
};
use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use sql_query_builder as sql;

// Structure that will implement the `MyRpcServer` trait.
// It can have fields, if required, as long as it's still `Send + Sync + 'static`.
pub(crate) struct RpcProxyImpl;

#[async_trait]
impl RpcProxyServer for RpcProxyImpl {
    async fn get_account_info(
        &self,
        base58_public_key: String,
        parameters: Option<Parameters>,
    ) -> RpcResult<String> {
        PubKey::parse(&base58_public_key)?;

        let encoding = Encoding::get_encoding(parameters.as_ref());

        match get_account_info(&base58_public_key, parameters.as_ref(), encoding).await? {
            None => Ok(json::Null.to_string()),
            Some(account) => {
                let owner = get_owner(&account.owner_id.to_string()).await?;

                Ok(account.to_json(encoding, &owner).to_string())
            }
        }
    }

    async fn get_multiple_accounts(
        &self,
        base58_public_keys: Vec<String>,
        _parameters: Option<Parameters>,
    ) -> RpcResult<String> {
        let public_keys: RpcResult<Vec<PubKey>> = base58_public_keys
            .iter()
            .map(|base58_public_key| PubKey::parse(base58_public_key))
            .collect();
        let _public_keys = public_keys?;

        Ok("getMultipleAccounts".into())
    }

    async fn get_program_accounts(
        &self,
        base58_public_key: String,
        _parameters: Option<Parameters>,
    ) -> RpcResult<String> {
        let _public_key = PubKey::parse(&base58_public_key)?;

        Ok("getProgramAccounts".into())
    }
}

/// Adds the [Commitment] level to the SQL subquery
pub fn commitment_subquery(commitment: &str) -> String {
    let mut commitment_subquery = String::new();
    commitment_subquery.push_str("(SELECT * FROM slot where status::VARCHAR = '");
    commitment_subquery.push_str(commitment);
    commitment_subquery.push_str("') slot_status ON account_write.slot = slot_status.slot");

    commitment_subquery
}

/// Adds a `WHERE` clause to match the public key in the field of `getAccountInfo` method
/// compares to any public key in the postgres database
pub fn where_pubkey(base58_public_key: &str) -> String {
    let mut pubkey_subquery = String::new();
    pubkey_subquery.push_str("pubkey = ");
    pubkey_subquery.push('\'');
    pubkey_subquery.push_str(base58_public_key);
    pubkey_subquery.push('\'');

    pubkey_subquery
}

/// The SQL query to fetch account information from the data store
pub async fn get_account_info(
    base58_public_key: &str,
    parameters: Option<&Parameters>,
    encoding: Encoding,
) -> RpcResult<Option<GetAccountInfoRow>> {
    let commitment = Commitment::get_commitment(parameters);

    let query = sql::Select::new()
            .select("pubkey.pubkey, account_write.slot, account_write.write_version, account_write.data, account_write.executable, account_write.owner_id, account_write.lamports, account_write.rent_epoch ")
            .from("account_write")
            .left_join("(SELECT pubkey.pubkey_id, pubkey.pubkey FROM pubkey) pubkey ON account_write.pubkey_id = pubkey.pubkey_id ")
            .left_join(&commitment_subquery(commitment))
            .where_clause(&where_pubkey(base58_public_key))
            .to_string();

    PgConnection::client_exists().await?;
    let guarded_pg_client = CLIENT.read().await;
    let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

    let rows = match pg_client.query(&query, &[]).await {
        Ok(value) => value,
        Err(error) => return Err(PgConnection::error_handler(&error)),
    };

    match rows.get(0) {
        None => Ok(None),
        Some(value) => {
            let row: GetAccountInfoRow = value.into();

            Ok(Some(row))
        }
    }
}

/// Fetches the public key from the `owner_id`
pub async fn get_owner(owner_id: &str) -> RpcResult<String> {
    let mut pubkey_subquery = String::new();
    pubkey_subquery.push_str("pubkey_id = ");
    pubkey_subquery.push('\'');
    pubkey_subquery.push_str(owner_id);
    pubkey_subquery.push('\'');

    let query = sql::Select::new()
        .select("pubkey.pubkey")
        .from("pubkey")
        .where_clause(&pubkey_subquery)
        .to_string();

    PgConnection::client_exists().await?;
    let guarded_pg_client = CLIENT.read().await;
    let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

    let rows = match pg_client.query(&query, &[]).await {
        Ok(value) => value,
        Err(error) => return Err(PgConnection::error_handler(&error)),
    };

    let owner: String = rows[0].get(0);

    Ok(owner)
}
