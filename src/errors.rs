use core::fmt;
use std::io::ErrorKind;

/// Handles `Result<_, Error>` for this crate
pub type RpcProxyResult<T> = Result<T, RpcProxyError>;

/// The errors supported by this crate.
#[derive(Debug)]
pub enum RpcProxyError {
    /// Errors occurring from `hyper` crate operations on network streams
    Hyper(String),
    /// An `std::io::Error` was encountered. It wraps  `std::io::ErrorKind`.
    Io(ErrorKind),
    /// Errors occurring from serializing or deserializing the JSON data from the
    /// HTTP body. This operation is handled by `serde_json` crate and wraps a `serde_json::Error`
    SerdeError(serde_json::Error),
}

impl fmt::Display for RpcProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RpcProxyError {}

impl From<std::io::Error> for RpcProxyError {
    fn from(error: std::io::Error) -> Self {
        RpcProxyError::Io(error.kind())
    }
}

impl From<hyper::Error> for RpcProxyError {
    fn from(error: hyper::Error) -> Self {
        RpcProxyError::Hyper(error.message().to_string())
    }
}

impl From<serde_json::Error> for RpcProxyError {
    fn from(error: serde_json::Error) -> Self {
        RpcProxyError::SerdeError(error)
    }
}
