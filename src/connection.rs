use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;

use crate::{to_napi_error, RecvStream, SendStream};

/// A QUIC connection to a remote endpoint
#[napi]
pub struct Connection {
    inner: Arc<iroh::endpoint::Connection>,
}

impl Connection {
    pub fn new(conn: iroh::endpoint::Connection) -> Self {
        Connection {
            inner: Arc::new(conn),
        }
    }
}

#[napi]
impl Connection {
    /// Get the remote endpoint's ID (public key)
    #[napi]
    pub fn remote_node_id(&self) -> String {
        self.inner.remote_id().to_string()
    }

    /// Get the ALPN protocol used for this connection
    #[napi]
    pub fn alpn(&self) -> Buffer {
        let alpn_slice: &[u8] = self.inner.alpn();
        Buffer::from(alpn_slice.to_vec())
    }

    /// Open a bidirectional stream
    /// Returns an object with send and recv properties
    #[napi]
    pub async fn open_bi(&self) -> Result<BiStreamResult> {
        let (send, recv) = self.inner.open_bi().await.map_err(to_napi_error)?;

        Ok(BiStreamResult {
            send: SendStream::new(send),
            recv: RecvStream::new(recv),
        })
    }

    /// Open a unidirectional stream (send only)
    #[napi]
    pub async fn open_uni(&self) -> Result<SendStream> {
        let send = self.inner.open_uni().await.map_err(to_napi_error)?;
        Ok(SendStream::new(send))
    }

    /// Accept a bidirectional stream opened by the remote
    #[napi]
    pub async fn accept_bi(&self) -> Result<BiStreamResult> {
        let (send, recv) = self.inner.accept_bi().await.map_err(to_napi_error)?;

        Ok(BiStreamResult {
            send: SendStream::new(send),
            recv: RecvStream::new(recv),
        })
    }

    /// Accept a unidirectional stream opened by the remote (receive only)
    #[napi]
    pub async fn accept_uni(&self) -> Result<RecvStream> {
        let recv = self.inner.accept_uni().await.map_err(to_napi_error)?;
        Ok(RecvStream::new(recv))
    }

    /// Close the connection with an error code and reason
    #[napi]
    pub fn close(&self, error_code: Option<u32>, reason: Option<String>) {
        let code = iroh::endpoint::VarInt::from_u32(error_code.unwrap_or(0));
        let reason_bytes = reason.unwrap_or_default().into_bytes();
        self.inner.close(code, &reason_bytes);
    }

    /// Wait for the connection to be closed
    #[napi]
    pub async fn closed(&self) -> Result<String> {
        let err = self.inner.closed().await;
        Ok(format!("{err}"))
    }

    /// Get the current round-trip time estimate in milliseconds
    #[napi]
    pub fn rtt(&self) -> f64 {
        self.inner.rtt().as_secs_f64() * 1000.0
    }

    /// Get the maximum datagram size that can be sent
    #[napi]
    pub fn max_datagram_size(&self) -> Option<u32> {
        self.inner.max_datagram_size().map(|s| s as u32)
    }

    /// Send an unreliable datagram
    #[napi]
    pub fn send_datagram(&self, data: Buffer) -> Result<()> {
        self.inner
            .send_datagram(bytes::Bytes::from(data.to_vec()))
            .map_err(to_napi_error)
    }

    /// Receive an unreliable datagram
    #[napi]
    pub async fn read_datagram(&self) -> Result<Buffer> {
        let data = self.inner.read_datagram().await.map_err(to_napi_error)?;
        Ok(Buffer::from(data.as_ref()))
    }
}

/// Result of opening a bidirectional stream
#[napi]
pub struct BiStreamResult {
    send: SendStream,
    recv: RecvStream,
}

#[napi]
impl BiStreamResult {
    #[napi(getter)]
    pub fn get_send(&self) -> SendStream {
        self.send.clone()
    }

    #[napi(getter)]
    pub fn get_recv(&self) -> RecvStream {
        self.recv.clone()
    }
}
