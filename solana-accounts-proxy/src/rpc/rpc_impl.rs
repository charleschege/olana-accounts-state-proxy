use crate::{
    Commitment, Context, CurrentSlot, DataSlice, Encoding, GetAccountInfoQuery, GetProgramAccounts,
    GetProgramAccountsRow, Parameters, PubKey, RpcProxyServer, WithContext,
};
use async_trait::async_trait;
use jsonrpsee::{core::Error as JsonrpseeError, core::RpcResult};
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

        Ok(get_program_accounts(&base58_public_key, parameters)
            .await?
            .into())
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

    let row = ga_query.query().await?;

    let mut query_result = Map::new();

    row.context.insert_json_value(&mut query_result);

    row.value.as_json_value(encoding, &mut query_result)?;

    Ok(Some(query_result.into()))
}

/// Handler the for `getProgramAccounts`
pub async fn get_program_accounts(
    base58_public_key: &str,
    parameters: Option<Parameters>,
) -> RpcResult<Option<JsonValue>> {
    dbg!(&parameters);

    let mut data_slice = DataSlice::default();
    let mut filters = Vec::default();
    let mut commitment = Commitment::Finalized;
    let mut min_context_slot: Option<u64> = Option::None;
    let encoding = Encoding::get_encoding(parameters.as_ref());

    let mut current_slot: Option<Context> = Option::None;

    if let Some(has_parameters) = parameters {
        if let Some(inner_data_slice) = has_parameters.data_slice.as_ref() {
            data_slice = *inner_data_slice;
        }
        if let Some(has_with_context) = has_parameters.with_context.as_ref() {
            if *has_with_context {
                current_slot = Some(
                    CurrentSlot::new()
                        .add_commitment(commitment)
                        .query()
                        .await?,
                );
            }
        }
        if let Some(has_filter) = has_parameters.filters {
            // Check if the number of `Filters` is greater than 4
            if has_filter.len() > 4 {
                return Err(JsonrpseeError::Custom(
                    "Too many filters provided; max 4".to_owned(),
                ));
            }
            filters = has_filter;
        }

        if let Some(req_commitment) = has_parameters.commitment {
            commitment = req_commitment;
        }

        min_context_slot = has_parameters.min_context_slot;
    }

    let gpa = GetProgramAccounts::new()
        .add_public_key(base58_public_key)
        .add_commitment(commitment.queryable())
        .add_min_context_slot(min_context_slot);

    let rows = gpa.basic_with_commitment().await?;

    let outcome = GetProgramAccountsRow::from_row(rows, encoding)?;

    if outcome.is_empty() {
        Ok(Option::None)
    } else if let Some(context) = current_slot {
        let with_context =
            WithContext::<Vec<JsonValue>>::new(context).as_json_value(outcome.into());

        Ok(Some(with_context.into()))
    } else {
        Ok(Some(outcome.into()))
    }
}
