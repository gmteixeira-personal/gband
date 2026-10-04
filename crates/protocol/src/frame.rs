use std::collections::VecDeque;

use serde::Serialize;
use serde::de::DeserializeOwned;

pub const HEADER_LEN: usize = 4;
pub const MAX_FRAME_LEN: usize = 16 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("frame of {0} bytes exceeds the {MAX_FRAME_LEN}-byte limit")]
    Oversized(usize),
    #[error("cannot encode message: {0}")]
    Encode(postcard::Error),
    #[error("cannot decode message: {0}")]
    Decode(postcard::Error),
    #[error("message is followed by {0} unexpected bytes")]
    TrailingBytes(usize),
}

pub fn encode<T: Serialize>(message: &T) -> Result<Vec<u8>, FrameError> {
    let mut frame =
        postcard::to_extend(message, vec![0; HEADER_LEN]).map_err(FrameError::Encode)?;
    let len = frame.len() - HEADER_LEN;
    if len > MAX_FRAME_LEN {
        return Err(FrameError::Oversized(len));
    }
    frame[..HEADER_LEN].copy_from_slice(&(len as u32).to_be_bytes());
    Ok(frame)
}

pub fn decode<T: DeserializeOwned>(payload: &[u8]) -> Result<T, FrameError> {
    match postcard::take_from_bytes(payload).map_err(FrameError::Decode)? {
        (message, []) => Ok(message),
        (_, rest) => Err(FrameError::TrailingBytes(rest.len())),
    }
}

pub fn leading_version(payload: &[u8]) -> Option<u32> {
    postcard::take_from_bytes::<u32>(payload)
        .ok()
        .map(|(version, _)| version)
}

#[derive(Debug, Default)]
pub struct Decoder {
    partial: Vec<u8>,
    payload_len: Option<usize>,
    payloads: VecDeque<Vec<u8>>,
}

impl Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, mut chunk: &[u8]) -> Result<(), FrameError> {
        loop {
            match self.payload_len {
                Some(len) if self.partial.len() == len => {
                    self.payloads.push_back(std::mem::take(&mut self.partial));
                    self.payload_len = None;
                }
                _ if chunk.is_empty() => return Ok(()),
                None => {
                    chunk = self.take(chunk, HEADER_LEN);
                    if self.partial.len() == HEADER_LEN {
                        let header: [u8; HEADER_LEN] = self.partial[..].try_into().unwrap();
                        let len = u32::from_be_bytes(header) as usize;
                        if len > MAX_FRAME_LEN {
                            return Err(FrameError::Oversized(len));
                        }
                        self.partial = Vec::with_capacity(len);
                        self.payload_len = Some(len);
                    }
                }
                Some(len) => chunk = self.take(chunk, len),
            }
        }
    }

    pub fn next_payload(&mut self) -> Option<Vec<u8>> {
        self.payloads.pop_front()
    }

    pub fn next_message<T: DeserializeOwned>(&mut self) -> Result<Option<T>, FrameError> {
        self.next_payload()
            .map(|payload| decode(&payload))
            .transpose()
    }

    fn take<'a>(&mut self, chunk: &'a [u8], target: usize) -> &'a [u8] {
        let (taken, rest) = chunk.split_at((target - self.partial.len()).min(chunk.len()));
        self.partial.extend_from_slice(taken);
        rest
    }
}
