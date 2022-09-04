use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

use crate::config::PostgresConfig;

/// A Postgres database connection
pub struct PgConnection {
    db: DatabaseConnection,
}

impl PgConnection {
    /// Connect to a database using the configuration [ProxyConfig]
    pub async fn connect(config: &PostgresConfig) -> anyhow::Result<PgConnection> {
        use secrecy::ExposeSecret;

        let mut connection_options =
            ConnectOptions::new(config.postgres_url().expose_secret().clone());

        if let Some(connections) = config.max_connections {
            connection_options.max_connections(connections);
        }

        if let Some(connections) = config.min_connections {
            connection_options.min_connections(connections);
        }

        if let Some(timeout) = config.connect_timeout {
            connection_options.connect_timeout(Duration::from_secs(timeout));
        }

        if let Some(timeout) = config.idle_timeout {
            connection_options.idle_timeout(Duration::from_secs(timeout));
        }

        if let Some(lifetime) = config.max_lifetime {
            connection_options.max_lifetime(Duration::from_secs(lifetime));
        }

        Ok(PgConnection {
            db: Database::connect(connection_options).await?,
        })
    }

    /// Get a reference to the database connection
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

/*
mod account_info;
pub use account_info::{Entity as AccountInfoEntity, Model as AccountInfoModel};

/// A test database
pub async fn check_mock_connection() {
    //let dbconn = Database::connect("postgres://root:root@localhost:5432").await?;

    let mock_dbconn = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results(vec![vec![AccountInfoModel {
            account_id: 1,
            key: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_owned(),
            is_signer: true,
            is_writable: true,
            lamports: 4_000_000_000,
            data: Vec::default(),
            owner: "11111111111111111111111111111111".to_owned(),
            executable: false,
            rent_epoch: 50,
        }]])
        .into_connection();
}

pub type Base58PublicKey = String;

*/
