use jsonrpsee::core::Error as JsonrpseeError;

/// Common conversion of stringified errors into a `jsonrpsee custom error`
#[derive(Debug)]
pub struct ErrorHandler {
    error: String,
}

impl ErrorHandler {
    /// Instantiate a new error handler
    pub fn new(error: &str) -> ErrorHandler {
        ErrorHandler {
            error: error.to_owned(),
        }
    }

    /// Build the error handler in a manner that it can be passed along by calling `?`
    /// on the build method
    pub fn build(self) -> JsonrpseeError {
        JsonrpseeError::Custom(self.error)
    }
}
