use crate::RpcProxyResult;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

// Loads the configuration to use on the server
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    ip: String,
    port: u16,
    tls: TlsConfig,
}

impl Config {
    pub(crate) fn load_file(path: &str) -> RpcProxyResult<Self> {
        let mut path_to_conf: PathBuf = path.into();
        path_to_conf.push("ProxyConfig.toml");

        let mut file = File::open(path_to_conf)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Config = toml::from_str(&contents)?;

        Ok(config)
    }

    pub(crate) fn get_socketaddr(&self) -> RpcProxyResult<SocketAddr> {
        let ipv4addr: Ipv4Addr = self.ip.parse()?;

        Ok(SocketAddr::new(IpAddr::V4(ipv4addr), self.port))
    }

    pub(crate) fn get_private(&self) -> PathBuf {
        self.tls.private.clone().into()
    }

    pub(crate) fn get_public(&self) -> PathBuf {
        self.tls.public.clone().into()
    }
}

// TLS specific configuration where `private` field represents the private key
// and the `public` field represents the public key certificate
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub(crate) struct TlsConfig {
    private: String,
    public: String,
}
