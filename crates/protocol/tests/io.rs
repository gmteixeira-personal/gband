use std::collections::VecDeque;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use gband_core::layout::PaneId;
use gband_protocol::{
    ClientMessage, FrameError, IoError, MAX_FRAME_LEN, MessageReader, MessageWriter, encode,
};
use tokio::io::{AsyncRead, ReadBuf};

struct Chunks(VecDeque<Vec<u8>>);

impl Chunks {
    fn new(chunks: impl IntoIterator<Item = Vec<u8>>) -> Self {
        Self(chunks.into_iter().collect())
    }
}

impl AsyncRead for Chunks {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if let Some(chunk) = self.0.pop_front() {
            buf.put_slice(&chunk);
        }
        Poll::Ready(Ok(()))
    }
}

fn paste(text: &str) -> ClientMessage {
    ClientMessage::Paste {
        pane: PaneId(1),
        text: text.to_owned(),
    }
}

#[tokio::test]
async fn message_split_across_reads_is_reassembled() {
    let frame = encode(&paste("split across reads")).unwrap();
    let chunks = frame.chunks(3).map(<[u8]>::to_vec);
    let mut reader = MessageReader::new(Chunks::new(chunks));
    let message: ClientMessage = reader.recv().await.unwrap().unwrap();
    assert_eq!(message, paste("split across reads"));
}

#[tokio::test]
async fn two_messages_from_one_read() {
    let frames = [
        encode(&paste("one")).unwrap(),
        encode(&paste("two")).unwrap(),
    ]
    .concat();
    let mut reader = MessageReader::new(Chunks::new([frames]));
    assert!(reader.fill().await.unwrap());
    assert_eq!(reader.try_recv().unwrap(), Some(paste("one")));
    assert_eq!(reader.try_recv().unwrap(), Some(paste("two")));
    assert_eq!(reader.try_recv::<ClientMessage>().unwrap(), None);
}

#[tokio::test]
async fn clean_end_of_stream_is_none() {
    let mut reader = MessageReader::new(Chunks::new([]));
    assert!(reader.recv::<ClientMessage>().await.unwrap().is_none());
}

#[tokio::test]
async fn oversized_frame_is_an_error() {
    let header = ((MAX_FRAME_LEN + 1) as u32).to_be_bytes().to_vec();
    let mut reader = MessageReader::new(Chunks::new([header]));
    let error = reader.recv::<ClientMessage>().await.unwrap_err();
    assert!(
        matches!(error, IoError::Frame(FrameError::Oversized(_))),
        "{error:?}"
    );
}

#[tokio::test]
async fn writer_output_reads_back() {
    let (near, far) = tokio::io::duplex(64);
    let mut writer = MessageWriter::new(near);
    let mut reader = MessageReader::new(far);
    let sending = tokio::spawn(async move {
        writer.send(&paste(&"x".repeat(1000))).await.unwrap();
    });
    let message: ClientMessage = reader.recv().await.unwrap().unwrap();
    assert_eq!(message, paste(&"x".repeat(1000)));
    sending.await.unwrap();
}
