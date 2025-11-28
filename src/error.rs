use napi::bindgen_prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IrohError {
    #[error("Bind error: {0}")]
    Bind(String),

    #[error("Connect error: {0}")]
    Connect(String),

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<IrohError> for napi::Error {
    fn from(err: IrohError) -> Self {
        napi::Error::from_reason(err.to_string())
    }
}

/// Convert any error to a napi::Error
pub fn to_napi_error<E: std::fmt::Display>(err: E) -> napi::Error {
    napi::Error::from_reason(err.to_string())
}
