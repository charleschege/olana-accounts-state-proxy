use core::fmt;
use std::{io::ErrorKind, net::AddrParseError};

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
    /// HTTP body. This operation is handled by `serde_json` crate
    SerdeJsonError(String),
    /// The Socket Address provided could not be parsed
    AddrParseError,
    /// A Custom Error
    Custom(String),
    /// The path to locate the config file was not provided as an argument when running the binary
    MissingPathToConfigFile,
    /// Error when parsing to an integer value
    Int(String),
}

impl fmt::Display for RpcProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RpcProxyError {}

impl From<AddrParseError> for RpcProxyError {
    fn from(_: AddrParseError) -> Self {
        RpcProxyError::AddrParseError
    }
}

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
        RpcProxyError::SerdeJsonError(error.to_string())
    }
}

impl From<core::num::ParseIntError> for RpcProxyError {
    fn from(error: core::num::ParseIntError) -> Self {
        RpcProxyError::Int(error.to_string())
    }
}
