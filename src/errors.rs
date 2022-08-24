use core::fmt;
use std::io::ErrorKind;

pub type RpcProxyResult<T> = Result<T, RpcProxyError>;

#[derive(Debug)]
pub enum RpcProxyError {
    /// Error can never happen, used in `hyper` crate to convert error using `?`
    Infallible,
    Io(ErrorKind),
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
    fn from(_: hyper::Error) -> Self {
        RpcProxyError::Infallible
    }
}

impl From<serde_json::Error> for RpcProxyError {
    fn from(error: serde_json::Error) -> Self {
        RpcProxyError::SerdeError(error)
    }
}
