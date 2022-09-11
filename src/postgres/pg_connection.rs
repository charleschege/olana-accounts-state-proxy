use crate::config::PostgresConfig;
use jsonrpsee::core::{Error as JsonrpseeError, RpcResult};
use std::time::Duration;
use tokio_postgres::{error::Severity, Client, Config, NoTls};

/// A Postgres database connection
pub struct PgConnection;

impl PgConnection {
    /// Connect to a database using the configuration [ProxyConfig]
    pub async fn connect(user_config: &PostgresConfig) -> anyhow::Result<Client> {
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

        let (client, db_conn) = config.connect(NoTls).await?;

        tokio::spawn(async {
            if let Err(error) = db_conn.await {
                tracing::error!("connection error: {}", error);

                std::process::exit(1)
            }
        });

        Ok(client)
    }

    /// Log all the errors encountered ny the Postgres connection
    /// and return a [jsonrpsee] compatible error
    pub fn error_handler(error: &tokio_postgres::Error) -> JsonrpseeError {
        match error.as_db_error() {
            Some(db_error) => {
                if let Some(severity) = db_error.parsed_severity() {
                    match severity {
                        Severity::Info => {
                            tracing::info!("connection error: {}", error.to_string());
                        }
                        Severity::Panic => {
                            tracing::trace!("connection error: {}", error.to_string());
                        }
                        Severity::Fatal => {
                            tracing::trace!("connection error: {}", error.to_string());
                        }
                        Severity::Error => {
                            tracing::error!("connection error: {}", error.to_string());
                        }
                        Severity::Warning => {
                            tracing::warn!("connection error: {}", error.to_string());
                        }
                        Severity::Notice => {
                            tracing::info!("connection error: {}", error.to_string());
                        }
                        Severity::Debug => {
                            tracing::debug!("connection error: {}", error.to_string());
                        }
                        Severity::Log => {
                            tracing::debug!("connection error: {}", error.to_string());
                        }
                    }
                } else {
                    PgConnection::unresolved_error(&error)
                }
            }
            None => PgConnection::unresolved_error(&error),
        }

        JsonrpseeError::Custom(
            "An internal server error occurred. Contact administrator or check the server logs."
                .to_owned(),
        )
    }

    /// Errors cannot be converted to [Severity]
    pub fn unresolved_error(error: &tokio_postgres::Error) {
        tracing::error!(
            "POSTGRES_ERROR_CODE `{:?}`. Error message `{:?}`",
            error.code(),
            error.to_string()
        );
    }

    /// Handles a HTTP response when the static variable [crate::CLIENT] is [Option::None]
    pub async fn client_exists() -> RpcResult<()> {
        if crate::CLIENT.read().await.is_none() {
            Err(JsonrpseeError::Custom(
                "Internal server error. The connection to the database does not exist.".to_owned(),
            ))
        } else {
            Ok(())
        }
    }
}
