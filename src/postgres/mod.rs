use std::time::Duration;

use crate::config::PostgresConfig;

/// A Postgres database connection
pub struct PgConnection {
    //db: DatabaseConnection,
}

impl PgConnection {
    /// Connect to a database using the configuration [ProxyConfig]
    pub async fn connect(config: &PostgresConfig) {
        //-> anyhow::Result<> {
        use secrecy::ExposeSecret;
    }

    /*/// Get a reference to the database connection
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }*/
}
