use secrecy::Secret;
use serde::{de, Deserialize, Serialize};
use std::{
    fmt,
    fs::File,
    io::Read,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

/// The configuration of the socket and database
#[derive(Debug, Deserialize)]
pub struct ProxyConfig {
    socket: SocketConfig,
    postgres: PostgresConfig,
}

impl ProxyConfig {
    // Load the configuration
    pub(crate) fn load_config(path: &str) -> anyhow::Result<Self> {
        let mut path_to_conf: PathBuf = path.into();
        path_to_conf.push("ProxyConfig.toml");

        let mut file = File::open(&path_to_conf)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: ProxyConfig = toml::from_str(&contents)?;

        Ok(config)
    }

    /// Computes the socket address of the IP and port from [ProxyConfig]
    pub fn get_socketaddr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(self.socket.ip), self.socket.port)
    }
}

/// Configuration specific to the IP address and port of the proxy server
#[derive(Debug, Serialize, Deserialize)]
pub struct SocketConfig {
    #[serde(deserialize_with = "ip_parser")]
    ip: Ipv4Addr,
    port: u16,
}

/// Configuration specific to the Postgres connection of the proxy server
#[derive(Deserialize)]
pub struct PostgresConfig {
    username: Secret<String>,
    password: Secret<String>,
    db_ip: String,
    db_name: Secret<String>,
    pub(crate) max_connections: Option<u32>,
    pub(crate) min_connections: Option<u32>,
    pub(crate) connect_timeout: Option<u64>,
    pub(crate) idle_timeout: Option<u64>,
    pub(crate) max_lifetime: Option<u64>,
}

impl PostgresConfig {
    /// Compute the postgres url `postgres://username:password@host/database`
    pub fn postgres_url(&self) -> Secret<String> {
        use secrecy::ExposeSecret;

        let mut url = "postgres://".to_owned();
        url.push_str(self.username.expose_secret());
        url.push(':');
        url.push_str(self.password.expose_secret());
        url.push('@');
        url.push_str(&self.db_ip);
        url.push('/');
        url.push_str(self.db_name.expose_secret());

        Secret::new(url)
    }
}

#[cfg(feature = "safe_debug")]
impl fmt::Debug for PostgresConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresConfig")
            .field("username", &"REDACTED[POSTGRES_USERNAME]")
            .field("password", &"REDACTED[POSTGRES_PASSWORD]")
            .field("db_ip", &self.db_ip)
            .field("db_name", &"REDACTED[POSTGRES_DATABASE]")
            .field("max_connections", &self.max_connections)
            .field("min_connections", &self.min_connections)
            .field("connect_timeout", &self.connect_timeout)
            .field("idle_timeout", &self.idle_timeout)
            .field("max_lifetime", &self.max_lifetime)
            .finish()
    }
}

#[cfg(feature = "dangerous_debug")]
impl fmt::Debug for PostgresConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use secrecy::ExposeSecret;

        f.debug_struct("PostgresConfig")
            .field("username", &self.username.expose_secret())
            .field("password", &self.password.expose_secret())
            .field("db_ip", &self.db_ip)
            .field("db_name", &self.db_name.expose_secret())
            .field("max_connections", &self.max_connections)
            .field("min_connections", &self.min_connections)
            .field("connect_timeout", &self.connect_timeout)
            .field("idle_timeout", &self.idle_timeout)
            .field("max_lifetime", &self.max_lifetime)
            .finish()
    }
}

fn ip_parser<'de, D>(deserializer: D) -> Result<Ipv4Addr, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct StringVisitor;

    impl<'de> de::Visitor<'de> for StringVisitor {
        type Value = Ipv4Addr;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("A string containing an IP address is required")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v.parse() {
                Ok(ip) => Ok(ip),
                Err(error) => Err(serde::de::Error::custom(error.to_string())),
            }
        }
    }

    deserializer.deserialize_any(StringVisitor)
}
