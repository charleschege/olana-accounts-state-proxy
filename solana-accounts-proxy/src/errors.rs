use jsonrpsee::core::Error as JsonrpseeError;

pub(crate) const INTERNAL_SERVER_ERROR: &str =
    "An error occured on the server. Contact admininstrator of check the server logs";

/// Result type for the crate
pub type ProxyResult<T> = Result<T, ProxyError>;

/// Error handler for the crate
#[derive(Debug)]
pub enum ProxyError {
    /// The error handler for `tokio_postgres` crate.
    /// Safely returns a predefined error to the client
    /// while logging the actual error to prevent leaking server information
    Pg(tokio_postgres::Error),
    /// An error that
    Client(String),
}

impl From<ProxyError> for jsonrpsee::core::Error {
    fn from(error: ProxyError) -> Self {
        match error {
            ProxyError::Pg(pg_error) => crate::PgConnection::error_handler(&pg_error),
            ProxyError::Client(safe_error) => JsonrpseeError::Custom(safe_error),
        }
    }
}

impl From<tokio_postgres::Error> for ProxyError {
    fn from(error: tokio_postgres::Error) -> Self {
        ProxyError::Pg(error)
    }
}
