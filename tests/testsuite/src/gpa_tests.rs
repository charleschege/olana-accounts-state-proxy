use crate::{TestsuiteConfig, APPLICATION_JSON, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use solana_accounts_proxy::{ProxyConfig, RpcResult, WithContext};
use std::path::Path;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, Clone)]
pub struct GetProgramAccountsTests {
    program_id: String,
    offset_public_key: String,
    offset: u64,
    data_size: u64,
    encoding: String,
    commitment: Option<String>,
    with_context: bool,
}

impl Default for GetProgramAccountsTests {
    fn default() -> Self {
        GetProgramAccountsTests::new()
    }
}

impl GetProgramAccountsTests {
    pub fn new() -> Self {
        GetProgramAccountsTests {
            program_id: String::default(),
            offset_public_key: String::default(),
            offset: u64::default(),
            data_size: u64::default(),
            encoding: String::default(),
            commitment: Option::None,
            with_context: false,
        }
    }

    pub fn add_program_id(&mut self, program_id: &str) -> &mut Self {
        self.program_id = program_id.to_owned();

        self
    }

    pub fn add_offset_public_key(&mut self, offset_public_key: &str) -> &mut Self {
        self.offset_public_key = offset_public_key.to_owned();

        self
    }

    pub fn add_offset(&mut self, offset: u64) -> &mut Self {
        self.offset = offset;

        self
    }

    pub fn add_data_size(&mut self, data_size: u64) -> &mut Self {
        self.data_size = data_size;

        self
    }

    pub fn add_encoding(&mut self, encoding: &str) -> &mut Self {
        self.encoding = encoding.to_owned();

        self
    }

    pub fn add_commitment(&mut self, commitment: &str) -> &mut Self {
        self.commitment = Some(commitment.to_owned());

        self
    }

    pub fn add_with_context(&mut self, with_context: bool) -> &mut Self {
        self.with_context = with_context;

        self
    }

    pub fn own(self) -> Self {
        self
    }

    pub fn to_json_string(&self) -> String {
        let commitment = match self.commitment.as_ref() {
            Some(commitment) => commitment.to_string(),
            None => "finalized".to_owned(),
        };

        json::object! {
            jsonrpc:"2.0",
            id: 1,
            method:"getProgramAccounts",
            params: json::array![
                self.program_id.to_string(),
                json::object!{
                    encoding: self.encoding.to_string(),
                    commitment: commitment.as_str(),
                    withContext: self.with_context
                }
            ]
        }
        .to_string()
    }

    pub async fn req_from_rpcpool(
        &self,
        config: &TestsuiteConfig,
    ) -> anyhow::Result<RpcResult<WithContext<Vec<RpcAccountInfo>>>> {
        let mainnet_url = config.url().clone();

        let response = minreq::post(mainnet_url)
            .with_header(CONTENT_TYPE, APPLICATION_JSON)
            .with_body(self.to_json_string())
            .send()?;

        Ok(serde_json::from_str::<
            RpcResult<WithContext<Vec<RpcAccountInfo>>>,
        >(response.as_str()?)?)
    }

    pub async fn req_from_proxy(
        &self,
        proxy_config_file: &Path,
    ) -> anyhow::Result<RpcResult<WithContext<Vec<RpcAccountInfo>>>> {
        let mut file = File::open(proxy_config_file).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let config = toml::from_str::<ProxyConfig>(&contents)?;
        tracing::info!("{:?}", &config);

        let mut proxy_url = String::new();
        proxy_url.push_str("http://");
        proxy_url.push_str(config.get_socketaddr().to_string().as_str());

        tracing::info!("PROXY ADDR FROM CONFIG FILE: {:?}", &proxy_url);

        let response = minreq::post(proxy_url)
            .with_header(CONTENT_TYPE, APPLICATION_JSON)
            .with_body(self.to_json_string())
            .send()?;

        Ok(serde_json::from_str::<
            RpcResult<WithContext<Vec<RpcAccountInfo>>>,
        >(response.as_str()?)?)
    }
}

/// AccountInfo which is just an [Account] with an additional field of `pubkey`
/// Account information
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct RpcAccountInfo {
    pub pubkey: String,
    pub account: RpcAccount,
}

/// An Account
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct RpcAccount {
    /// The data specific to the account in the specified encoding format `(data, encoding_format)`
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
