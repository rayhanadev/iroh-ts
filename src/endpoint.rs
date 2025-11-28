use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;

use crate::{to_napi_error, Connection};

/// Options for creating an Endpoint
#[napi(object)]
#[derive(Default)]
pub struct EndpointOptions {
    /// ALPN protocols to accept for incoming connections
    pub alpns: Option<Vec<String>>,
    /// Secret key as hex string (32 bytes). If not provided, a random key is generated.
    pub secret_key: Option<String>,
}

/// An iroh Endpoint for establishing p2p connections
#[napi]
pub struct Endpoint {
    inner: Arc<iroh::Endpoint>,
}

#[napi]
impl Endpoint {
    /// Create a new Endpoint with default settings
    #[napi(factory)]
    pub async fn create() -> Result<Endpoint> {
        let ep = iroh::Endpoint::builder()
            .bind()
            .await
            .map_err(to_napi_error)?;

        Ok(Endpoint {
            inner: Arc::new(ep),
        })
    }

    /// Create a new Endpoint with options
    #[napi(factory)]
    pub async fn create_with_options(options: EndpointOptions) -> Result<Endpoint> {
        let mut builder = iroh::Endpoint::builder();

        // Set secret key if provided
        if let Some(ref key_hex) = options.secret_key {
            let key_bytes = hex_to_bytes(key_hex).map_err(to_napi_error)?;
            if key_bytes.len() != 32 {
                return Err(napi::Error::from_reason(
                    "Secret key must be 32 bytes (64 hex chars)",
                ));
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&key_bytes);
            let secret_key = iroh::SecretKey::from(arr);
            builder = builder.secret_key(secret_key);
        }

        // Set ALPNs if provided
        if let Some(alpns) = options.alpns {
            let alpn_bytes: Vec<Vec<u8>> = alpns.into_iter().map(|s| s.into_bytes()).collect();
            builder = builder.alpns(alpn_bytes);
        }

        let ep = builder.bind().await.map_err(to_napi_error)?;

        Ok(Endpoint {
            inner: Arc::new(ep),
        })
    }

    /// Get the endpoint's unique identifier (public key)
    #[napi]
    pub fn node_id(&self) -> String {
        self.inner.id().to_string()
    }

    /// Get the endpoint's address information as a string
    /// This can be shared with others to allow them to connect
    #[napi]
    pub fn addr(&self) -> String {
        format!("{:?}", self.inner.addr())
    }

    /// Wait until the endpoint is online (connected to a relay)
    #[napi]
    pub async fn online(&self) -> Result<()> {
        self.inner.online().await;
        Ok(())
    }

    /// Connect to a remote endpoint
    ///
    /// @param addr - The remote endpoint address (NodeId or full EndpointAddr)
    /// @param alpn - The ALPN protocol to use for this connection
    #[napi]
    pub async fn connect(&self, addr: String, alpn: String) -> Result<Connection> {
        // Parse the address as an EndpointId first (just the node ID)
        let endpoint_id: iroh::EndpointId = addr
            .parse()
            .map_err(|e| to_napi_error(format!("Invalid node ID: {e}")))?;

        let conn = self
            .inner
            .connect(endpoint_id, alpn.as_bytes())
            .await
            .map_err(to_napi_error)?;

        Ok(Connection::new(conn))
    }

    /// Accept an incoming connection
    /// Returns None if the endpoint is closed
    #[napi]
    pub async fn accept(&self) -> Result<Option<Connection>> {
        match self.inner.accept().await {
            Some(incoming) => {
                let conn = incoming.await.map_err(to_napi_error)?;
                Ok(Some(Connection::new(conn)))
            }
            None => Ok(None),
        }
    }

    /// Close the endpoint gracefully
    #[napi]
    pub async fn close(&self) -> Result<()> {
        self.inner.close().await;
        Ok(())
    }

    /// Check if the endpoint is closed
    #[napi]
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

/// Helper to convert hex string to bytes
fn hex_to_bytes(hex: &str) -> std::result::Result<Vec<u8>, String> {
    if !hex.len().is_multiple_of(2) {
        return Err("Hex string must have even length".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at position {i}: {e}"))
        })
        .collect()
}
