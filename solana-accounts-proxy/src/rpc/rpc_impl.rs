use crate::{
    Commitment, DataSlice, Encoding, Filter, GetAccountInfoQuery, GetAccountInfoRow, Parameters,
    PgConnection, PubKey, RpcProxyServer, CLIENT,
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
        parameters: Option<Parameters>,
    ) -> RpcResult<serde_json::Value> {
        let _public_key = PubKey::parse(&base58_public_key)?;

        if let Some(parameters) = parameters.as_ref() {
            parameters.exceeds_filters_len()?;
        }

        dbg!(&parameters);

        get_program_accounts(&base58_public_key, parameters).await;

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

/// The handler for `getAccountInfo` method
pub async fn get_account_info(
    base58_public_key: &str,
    parameters: Option<&Parameters>,
) -> RpcResult<Option<JsonValue>> {
    let commitment = Commitment::get_commitment(parameters);
    let encoding = Encoding::get_encoding(parameters);

    let mut ga_query = GetAccountInfoQuery::new();
    ga_query
        .add_public_key(base58_public_key)
        .add_commitment(commitment);

    let mut offset = 0usize;
    let mut offset_length = 0usize;

    if let Some(parameters_inner) = parameters {
        ga_query.add_min_context_slot(parameters_inner.min_context_slot);

        if let Some(data_slice_inner) = parameters_inner.data_slice {
            offset = data_slice_inner.offset;
            offset_length = data_slice_inner.length;
        }
    }

    let query = ga_query.query();

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
            let mut row: GetAccountInfoRow = value.into();

            row.value.as_data_slice(offset, offset_length);

            let mut query_result = Map::new();

            row.context.as_json_value(&mut query_result);
            row.value.as_json_value(encoding, &mut query_result)?;

            Ok(Some(query_result.into()))
        }
    }
}

/// Handler the for `getProgramAccounts`
pub async fn get_program_accounts(base58_public_key: &str, parameters: Option<Parameters>) {
    let mut data_slice = DataSlice::default();
    let mut with_context = bool::default();
    let mut filters = Vec::<Filter>::default();

    if let Some(has_parameters) = parameters {
        if let Some(inner_data_slice) = has_parameters.data_slice.as_ref() {
            data_slice = *inner_data_slice;
        }
        if let Some(has_with_context) = has_parameters.with_context.as_ref() {
            with_context = *has_with_context;
        }
        if let Some(has_filter) = has_parameters.filters {
            filters = has_filter;
        }
    }

    dbg!(&data_slice);
    dbg!(&with_context);
    dbg!(&filters);
}
