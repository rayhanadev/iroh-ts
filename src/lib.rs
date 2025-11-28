#![deny(clippy::all)]

use napi_derive::napi;

mod connection;
mod endpoint;
mod error;
mod stream;

pub use connection::*;
pub use endpoint::*;
pub use error::*;
pub use stream::*;

/// Initialize logging (optional, can be called from JS)
#[napi]
pub fn init_logging(level: Option<String>) {
    let filter = level.unwrap_or_else(|| "info".to_string());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
