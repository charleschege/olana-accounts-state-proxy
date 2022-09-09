use crate::config::PostgresConfig;
use std::time::Duration;
use tokio_postgres::{tls::NoTlsStream, Client, Config, Connection, NoTls, Socket};
/// A Postgres database connection
pub struct PgConnection;

impl PgConnection {
    /// Connect to a database using the configuration [ProxyConfig]
    pub async fn connect(
        user_config: &PostgresConfig,
    ) -> anyhow::Result<(Client, Connection<Socket, NoTlsStream>)> {
        use secrecy::ExposeSecret;

        let mut config = Config::new();
        config
            .user(user_config.user.expose_secret())
            .dbname(&user_config.dbname)
            .host(user_config.host.as_str());

        if let Some(password) = &user_config.password {
            config.password(password.expose_secret());
        }

        if let Some(port) = user_config.port {
            config.port(port);
        }

        if let Some(options) = &user_config.options {
            config.options(options);
        }

        if let Some(application_name) = &user_config.application_name {
            config.application_name(application_name);
        }

        if let Some(secs) = user_config.connect_timeout {
            config.connect_timeout(Duration::from_secs(secs));
        }

        Ok(config.connect(NoTls).await?)
    }
}
