use crate::{TestsuiteConfig, APPLICATION_JSON, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use solana_accounts_proxy::{AccountInfo, Context, ProxyConfig, RpcResult, RpcResultData};
use std::{borrow::Cow, path::Path};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug)]
pub struct GetProgramAccountsTests<'gpa> {
    program_id: Cow<'gpa, str>,
    offset_public_key: Cow<'gpa, str>,
    offset: u64,
    data_size: u64,
    encoding: Cow<'gpa, str>,
}

impl<'gpa> GetProgramAccountsTests<'gpa> {
    pub fn new() -> Self {
        GetProgramAccountsTests {
            program_id: Cow::default(),
            offset_public_key: Cow::default(),
            offset: u64::default(),
            data_size: u64::default(),
            encoding: Cow::default(),
        }
    }

    pub fn add_program_id(&mut self, program_id: &'gpa str) -> &mut Self {
        self.program_id = Cow::Borrowed(program_id);

        self
    }

    pub fn add_offset_public_key(&mut self, offset_public_key: &'gpa str) -> &mut Self {
        self.offset_public_key = Cow::Borrowed(offset_public_key);

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

    pub fn add_encoding(&mut self, encoding: &'gpa str) -> &mut Self {
        self.encoding = Cow::Borrowed(encoding);

        self
    }

    pub fn own(self) -> Self {
        self
    }

    pub fn to_json_string(&self) -> String {
        json::object! {
            jsonrpc:"2.0",
            id: 1,
            method:"getProgramAccounts",
            params: json::array![
                self.program_id.to_string(),
                json::object!{
                    encoding: self.encoding.to_string(),
                    filters: json::array![
                        json::object!{ dataSize: self.data_size },
                        json::object!{
                            memcmp: json::object!{
                                offset: self.offset,
                                bytes: self.offset_public_key.to_string(),
                            }
                        }
                    ]
                }
            ]
        }
        .to_string()
    }

    pub async fn req_from_rpcpool(
        &self,
        config: &TestsuiteConfig,
    ) -> anyhow::Result<RpcResult<Vec<RpcAccountInfo>>> {
        let mainnet_url = config.url().clone();

        let response = minreq::post(mainnet_url)
            .with_header(CONTENT_TYPE, APPLICATION_JSON)
            .with_body(self.to_json_string())
            .send()?;

        Ok(serde_json::from_str::<RpcResult<Vec<RpcAccountInfo>>>(
            response.as_str()?,
        )?)
    }

    pub async fn req_from_proxy(
        &self,
        proxy_config_file: &Path,
    ) -> anyhow::Result<RpcResult<Vec<RpcAccountInfo>>> {
        let mut file = File::open(proxy_config_file).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let config = toml::from_str::<ProxyConfig>(&contents)?;

        dbg!(&config);

        let mut proxy_url = String::new();
        proxy_url.push_str("http://");
        proxy_url.push_str(config.get_socketaddr().to_string().as_str());

        let response = minreq::post(proxy_url)
            .with_header(CONTENT_TYPE, APPLICATION_JSON)
            .with_body(self.to_json_string())
            .send()?;

        dbg!(&response.as_str());

        Ok(serde_json::from_str::<RpcResult<Vec<RpcAccountInfo>>>(
            response.as_str()?,
        )?)
    }
}

/// AccountInfo which is just an [Account] with an additional field of `pubkey`
/// Account information
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcAccountInfo {
    pubkey: String,
    account: RpcAccount,
}

/// An Account
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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
