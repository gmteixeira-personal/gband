use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::frame::{Decoder, FrameError, encode};

const READ_BUFFER_LEN: usize = 64 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error(transparent)]
    Frame(#[from] FrameError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

pub struct MessageReader<R> {
    reader: R,
    decoder: Decoder,
    buffer: Vec<u8>,
}

impl<R: AsyncRead + Unpin> MessageReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            decoder: Decoder::new(),
            buffer: vec![0; READ_BUFFER_LEN],
        }
    }

    pub async fn recv<T: DeserializeOwned>(&mut self) -> Result<Option<T>, IoError> {
        loop {
            if let Some(message) = self.try_recv()? {
                return Ok(Some(message));
            }
            if !self.fill().await? {
                return Ok(None);
            }
        }
    }

    pub async fn recv_payload(&mut self) -> Result<Option<Vec<u8>>, IoError> {
        loop {
            if let Some(payload) = self.decoder.next_payload() {
                return Ok(Some(payload));
            }
            if !self.fill().await? {
                return Ok(None);
            }
        }
    }

    pub fn try_recv<T: DeserializeOwned>(&mut self) -> Result<Option<T>, IoError> {
        Ok(self.decoder.next_message()?)
    }

    pub async fn fill(&mut self) -> Result<bool, IoError> {
        let n = self.reader.read(&mut self.buffer).await?;
        if n == 0 {
            return Ok(false);
        }
        self.decoder.feed(&self.buffer[..n])?;
        Ok(true)
    }
}

pub struct MessageWriter<W> {
    writer: W,
}

impl<W: AsyncWrite + Unpin> MessageWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    pub async fn send<T: Serialize>(&mut self, message: &T) -> Result<(), IoError> {
        self.writer.write_all(&encode(message)?).await?;
        self.writer.flush().await?;
        Ok(())
    }
}
