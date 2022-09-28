use crate::{
    Commitment, Encoding, GetAccountInfoQuery, GetAccountInfoRow, Parameters, PgConnection, PubKey,
    RpcProxyServer, CLIENT,
};
use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use serde_json::{Map, Value as JsonValue};

// Structure that will implement the `MyRpcServer` trait.
// It can have fields, if required, as long as it's still `Send + Sync + 'static`.
pub(crate) struct RpcProxyImpl;

#[async_trait]
impl RpcProxyServer for RpcProxyImpl {
    async fn get_account_info(
        &self,
        base58_public_key: String,
        parameters: Option<Parameters>,
    ) -> RpcResult<Option<JsonValue>> {
        PubKey::parse(&base58_public_key)?;

        get_account_info(&base58_public_key, parameters.as_ref()).await
    }

    async fn get_program_accounts(
        &self,
        base58_public_key: String,
        _parameters: Option<Parameters>,
    ) -> RpcResult<serde_json::Value> {
        let _public_key = PubKey::parse(&base58_public_key)?;

        Ok("getProgramAccounts".into())
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
}

/// The SQL query to fetch account information from the data store
pub async fn get_account_info(
    base58_public_key: &str,
    parameters: Option<&Parameters>,
) -> RpcResult<Option<JsonValue>> {
    let commitment = Commitment::get_commitment(parameters);
    let encoding = Encoding::get_encoding(parameters);

    let query = GetAccountInfoQuery::new()
        .add_public_key(base58_public_key)
        .add_commitment(commitment)
        .query();

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

            let mut query_result = Map::new();

            row.context.as_json_value(&mut query_result);
            row.value.as_json_value(encoding, &mut query_result)?;

            Ok(Some(query_result.into()))
        }
    }
}
