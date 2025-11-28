use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::to_napi_error;

/// A stream for sending data to a remote endpoint
#[derive(Clone)]
#[napi]
pub struct SendStream {
    inner: Arc<Mutex<iroh::endpoint::SendStream>>,
}

impl SendStream {
    pub fn new(stream: iroh::endpoint::SendStream) -> Self {
        SendStream {
            inner: Arc::new(Mutex::new(stream)),
        }
    }
}

#[napi]
impl SendStream {
    /// Write data to the stream
    #[napi]
    pub async fn write(&self, data: Buffer) -> Result<u32> {
        let mut stream = self.inner.lock().await;
        let written = stream.write(&data).await.map_err(to_napi_error)?;
        Ok(written as u32)
    }

    /// Write all data to the stream
    #[napi]
    pub async fn write_all(&self, data: Buffer) -> Result<()> {
        let mut stream = self.inner.lock().await;
        stream.write_all(&data).await.map_err(to_napi_error)
    }

    /// Finish the stream, signaling no more data will be sent
    #[napi]
    pub async fn finish(&self) -> Result<()> {
        let mut stream = self.inner.lock().await;
        stream.finish().map_err(to_napi_error)
    }

    /// Reset the stream with an error code
    #[napi]
    pub async fn reset(&self, error_code: u32) -> Result<()> {
        let mut stream = self.inner.lock().await;
        stream
            .reset(iroh::endpoint::VarInt::from_u32(error_code))
            .map_err(to_napi_error)
    }

    /// Get the stream ID
    #[napi]
    pub async fn id(&self) -> String {
        let stream = self.inner.lock().await;
        format!("{}", stream.id())
    }
}

/// A stream for receiving data from a remote endpoint
#[derive(Clone)]
#[napi]
pub struct RecvStream {
    inner: Arc<Mutex<iroh::endpoint::RecvStream>>,
}

impl RecvStream {
    pub fn new(stream: iroh::endpoint::RecvStream) -> Self {
        RecvStream {
            inner: Arc::new(Mutex::new(stream)),
        }
    }
}

#[napi]
impl RecvStream {
    /// Read data from the stream
    /// Returns null when the stream is finished
    #[napi]
    pub async fn read(&self, max_length: u32) -> Result<Option<Buffer>> {
        let mut stream = self.inner.lock().await;
        let mut buf = vec![0u8; max_length as usize];
        match stream.read(&mut buf).await {
            Ok(Some(n)) => {
                buf.truncate(n);
                Ok(Some(Buffer::from(buf)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(to_napi_error(e)),
        }
    }

    /// Read exactly the specified number of bytes
    #[napi]
    pub async fn read_exact(&self, length: u32) -> Result<Buffer> {
        let mut stream = self.inner.lock().await;
        let mut buf = vec![0u8; length as usize];
        stream.read_exact(&mut buf).await.map_err(to_napi_error)?;
        Ok(Buffer::from(buf))
    }

    /// Read all remaining data from the stream up to a maximum size
    #[napi]
    pub async fn read_to_end(&self, max_length: u32) -> Result<Buffer> {
        let mut stream = self.inner.lock().await;
        let result = stream
            .read_to_end(max_length as usize)
            .await
            .map_err(to_napi_error)?;
        Ok(Buffer::from(result.as_ref()))
    }

    /// Stop reading from the stream
    #[napi]
    pub async fn stop(&self, error_code: u32) -> Result<()> {
        let mut stream = self.inner.lock().await;
        stream
            .stop(iroh::endpoint::VarInt::from_u32(error_code))
            .map_err(to_napi_error)
    }

    /// Get the stream ID
    #[napi]
    pub async fn id(&self) -> String {
        let stream = self.inner.lock().await;
        format!("{}", stream.id())
    }
}
