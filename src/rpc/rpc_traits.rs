use crate::Parameters;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[rpc(server, namespace = "proxy")]
pub trait RpcProxy {
    /// Processes the `getAccountInfo` method
    #[method(name = "getAccountInfo", aliases = ["getAccountInfo"])]
    async fn get_account_info(
        &self,
        public_key: String,
        parameters: Option<Parameters>,
    ) -> RpcResult<String>;

    /// Processes the `getAccountInfo` method
    #[method(name = "getMultipleAccounts", aliases = ["getMultipleAccounts"])]
    async fn get_multiple_accounts(
        &self,
        public_keys: Vec<String>,
        parameters: Option<Parameters>,
    ) -> RpcResult<String>;

    /// Get all accounts owned by the provided public key
    #[method(name = "getProgramAccounts", aliases = ["getProgramAccounts"])]
    async fn get_program_accounts(
        &self,
        public_key: String,
        parameters: Option<Parameters>,
    ) -> RpcResult<String>;
}
