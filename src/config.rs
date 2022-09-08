use secrecy::Secret;
use serde::{de, Deserialize, Serialize};
use std::{
    fmt,
    fs::File,
    io::Read,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};
use tokio_postgres::config::{ChannelBinding, SslMode};

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

    /// Load postgres configuration
    pub(crate) fn postgres_config(&self) -> &PostgresConfig {
        &self.postgres
    }
}

/// Configuration specific to the IP address and port of the proxy server
#[derive(Debug, Serialize, Deserialize)]
pub struct SocketConfig {
    #[serde(deserialize_with = "ip_parser")]
    ip: Ipv4Addr,
    port: u16,
}

/// The configuration to pass to the Postgres connection
#[derive(Deserialize)]
pub struct PostgresConfig {
    pub(crate) user: Secret<String>,
    pub(crate) password: Option<Secret<String>>,
    pub(crate) dbname: Secret<String>,
    pub(crate) host: String,
    pub(crate) port: Option<u16>,
    // Command line options to pass to the Postgres server
    pub(crate) options: Option<String>,
    //  Sets the application name to be reported in statistics and logs
    pub(crate) application_name: Option<String>,
    // `Require`, `Prefer` or `Disable`
    pub(crate) ssl_mode: Option<String>,
    pub(crate) connect_timeout: Option<u64>,
    // Channel binding is a concept defined in RFC 5056,
    // to ensure that the frontend and the backend connecting to each other are the same
    // in order to prevent man-in-the-middle attacks.
    // `Require`, `Prefer` or `Disable`
    pub(crate) channel_binding: Option<String>,
}

impl PostgresConfig {
    /// Get the SSL mode by converting the configuration to [SslMode]
    pub fn get_ssl_mode(&self) -> SslMode {
        if let Some(ssl_mode) = &self.ssl_mode {
            match ssl_mode.to_lowercase().as_str() {
                "prefer" => SslMode::Prefer,
                "disable" => SslMode::Disable,
                _ => SslMode::Require,
            }
        } else {
            SslMode::Require
        }
    }

    /// Get the channel binding mode by converting the configuration to [ChannelBinding]
    pub fn get_channel_binding(&self) -> ChannelBinding {
        if let Some(channel_binding) = &self.channel_binding {
            match channel_binding.to_lowercase().as_str() {
                "prefer" => ChannelBinding::Prefer,
                "disable" => ChannelBinding::Disable,
                _ => ChannelBinding::Require,
            }
        } else {
            ChannelBinding::Require
        }
    }

    /// Compute the postgres url `postgres://username:password@host/database`
    #[cfg(all(debug_assertions, feature = "dangerous_debug"))]
    pub fn postgres_url(&self) -> String {
        use secrecy::ExposeSecret;

        let password = match &self.password {
            Some(password) => {
                let mut password_formatting = String::from(":");
                password_formatting.push_str(password.expose_secret());

                password_formatting
            }
            None => "".to_owned(),
        };

        let mut url = "postgres://".to_owned();
        url.push_str(self.user.expose_secret());
        url.push_str(&password);
        url.push('@');
        url.push_str(&self.host);
        url.push('/');
        url.push_str(self.dbname.expose_secret());

        url
    }
}

#[cfg(feature = "safe_debug")]
impl fmt::Debug for PostgresConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresConfig")
            .field("user", &"REDACTED[POSTGRES_USER]")
            .field("password", {
                if self.password.is_some() {
                    &Some(&"REDACTED[POSTGRES_PASSWORD]")
                } else {
                    &Option::<String>::None
                }
            })
            .field("dbname", &"REDACTED[POSTGRES_DATABASE]")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("options", &self.options)
            .field("application_name", &self.application_name)
            .field("ssl_mode", &self.ssl_mode)
            .field("connect_timeout", &self.connect_timeout)
            .field("channel_binding", &self.channel_binding)
            .finish()
    }
}

#[cfg(feature = "dangerous_debug")]
impl fmt::Debug for PostgresConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use secrecy::ExposeSecret;

        f.debug_struct("PostgresConfig")
            .field("user", &self.user.expose_secret())
            .field("password", &{
                if let Some(password) = &self.password {
                    Some(password.expose_secret())
                } else {
                    Option::<&String>::None
                }
            })
            .field("dbname", &self.dbname.expose_secret())
            .field("host", &self.host)
            .field("port", &self.port)
            .field("options", &self.options)
            .field("application_name", &self.application_name)
            .field("ssl_mode", &self.ssl_mode)
            .field("connect_timeout", &self.connect_timeout)
            .field("channel_binding", &self.channel_binding)
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
