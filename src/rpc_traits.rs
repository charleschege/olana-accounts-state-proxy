use async_trait::async_trait;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use std::collections::HashMap;

#[rpc(server, namespace = "proxy")]
pub trait RpcProxy {
    /// Processes the `getAccountInfo` method
    #[method(name = "getAccountInfo", aliases = ["getAccountInfo"])]
    async fn get_account_info(
        &self,
        public_key: String,
        parameters: HashMap<String, String>,
    ) -> RpcResult<String>;
}

// Structure that will implement the `MyRpcServer` trait.
// It can have fields, if required, as long as it's still `Send + Sync + 'static`.
pub(crate) struct RpcProxyImpl;

#[async_trait]
impl RpcProxyServer for RpcProxyImpl {
    async fn get_account_info(
        &self,
        public_key: String,
        parameters: HashMap<String, String>,
    ) -> RpcResult<String> {
        Ok("GREAT_WORK".into())
    }
}
