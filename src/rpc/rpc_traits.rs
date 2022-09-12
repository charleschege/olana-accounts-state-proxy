use crate::Parameters;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use serde::Serialize;

#[rpc(server, namespace = "proxy")]
pub trait RpcProxy {
    /// Processes the `getAccountInfo` method
    #[method(name = "getAccountInfo", aliases = ["getAccountInfo"])]
    async fn get_account_info(
        &self,
        public_key: String,
        parameters: Option<Parameters>,
    ) -> RpcResult<Option<GetAccountInfo>>;

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

/// The result of an rpc query
#[derive(Debug, Serialize)]
pub struct GetAccountInfo {
    /// Information about the slot and rpc server
    pub context: Context,
    /// The data specific to an account
    pub value: RpcValue,
}

/// Data specific to an account
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcValue {
    /// The data specific to the account
    pub data: (String, String),
    /// Is the account executable
    pub executable: bool,
    /// Number of lamports held by the account
    pub lamports: i64,
    /// The owner of the account
    pub owner: String,
    /// Next epoch when rent will be collected
    pub rent_epoch: i64,
}

/// Slot context
#[derive(Debug, Serialize)]
pub struct Context {
    /// The period of time for which each leader ingests transactions and produces a block.
    pub slot: i64,
}
