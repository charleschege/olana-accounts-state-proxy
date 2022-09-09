use crate::{Parameters, PubKey, RpcProxyServer};
use async_trait::async_trait;
use jsonrpsee::core::RpcResult;

// Structure that will implement the `MyRpcServer` trait.
// It can have fields, if required, as long as it's still `Send + Sync + 'static`.
pub(crate) struct RpcProxyImpl;

#[async_trait]
impl RpcProxyServer for RpcProxyImpl {
    async fn get_account_info(
        &self,
        base58_public_key: String,
        _parameters: Option<Parameters>,
    ) -> RpcResult<String> {
        let _public_key = PubKey::parse(&base58_public_key)?;

        Ok("getAccountInfo".into())
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
