use crate::{RpcAccount, TestsuiteConfig, APPLICATION_JSON, CONTENT_TYPE};
use solana_accounts_proxy::{ProxyConfig, RpcResult, WithContext};
use std::{borrow::Cow, path::Path};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, Default, Clone)]
pub struct Ga<'gpa> {
    pubkey: Cow<'gpa, str>,
    encoding: Cow<'gpa, str>,
    commitment: Option<Cow<'gpa, str>>,
}

impl<'gpa> Ga<'gpa> {
    pub fn new() -> Self {
        Ga::default()
    }

    pub fn add_pubkey(mut self, pubkey: &'gpa str) -> Self {
        self.pubkey = Cow::Borrowed(pubkey);

        self
    }

    pub fn add_encoding(mut self, encoding: &'gpa str) -> Self {
        self.encoding = Cow::Borrowed(encoding);

        self
    }

    pub fn add_commitment(mut self, commitment: &'gpa str) -> Self {
        self.commitment = Some(Cow::Borrowed(commitment));

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
            method:"getAccountInfo",
            params: json::array![
                self.pubkey.to_string(),
                json::object!{
                    encoding: self.encoding.to_string(),
                    commitment: commitment.as_str(),
                }
            ]
        }
        .to_string()
    }

    pub async fn req_from_rpcpool(
        &self,
        config: TestsuiteConfig,
    ) -> anyhow::Result<RpcResult<WithContext<RpcAccount>>> {
        let mainnet_url = config.url().clone();

        let response = minreq::post(mainnet_url)
            .with_header(CONTENT_TYPE, APPLICATION_JSON)
            .with_body(self.to_json_string())
            .send()?;

        Ok(serde_json::from_str::<RpcResult<WithContext<RpcAccount>>>(
            response.as_str()?,
        )?)
    }

    pub async fn req_from_proxy(
        &self,
        proxy_config_file: &Path,
    ) -> anyhow::Result<RpcResult<WithContext<RpcAccount>>> {
        let mut file = File::open(proxy_config_file).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let config = toml::from_str::<ProxyConfig>(&contents)?;

        let mut proxy_url = String::new();
        proxy_url.push_str("http://");
        proxy_url.push_str(config.get_socketaddr().to_string().as_str());

        let response = minreq::post(proxy_url)
            .with_header(CONTENT_TYPE, APPLICATION_JSON)
            .with_body(self.to_json_string())
            .send()?;

        Ok(serde_json::from_str::<RpcResult<WithContext<RpcAccount>>>(
            response.as_str()?,
        )?)
    }
}
