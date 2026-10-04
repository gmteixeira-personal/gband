use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context as _, Result};
use gband_protocol::{
    ClientMessage, Decoder, Hello, HelloReply, PROTOCOL_VERSION, ServerMessage, encode,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::{mpsc, watch};

use crate::pane::{Input, Pane};

const READ_BUFFER_LEN: usize = 64 * 1024;

pub struct Context {
    pub pane: Arc<Pane>,
    pub input: mpsc::UnboundedSender<Input>,
    pub ended: watch::Receiver<bool>,
    pub info: ServerMessage,
}

pub async fn serve(stream: UnixStream, context: Arc<Context>) {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    tracing::debug!(client = id, "client connected");
    match handle(stream, &context).await {
        Ok(()) => tracing::debug!(client = id, "client disconnected"),
        Err(error) => tracing::warn!(client = id, "closing client connection: {error:#}"),
    }
}

async fn handle(stream: UnixStream, context: &Context) -> Result<()> {
    let (mut reader, mut writer) = stream.into_split();
    let mut decoder = Decoder::new();
    let mut buffer = vec![0; READ_BUFFER_LEN];

    let Some(hello) = read_message::<Hello>(&mut reader, &mut decoder, &mut buffer)
        .await
        .context("invalid hello")?
    else {
        return Ok(());
    };
    if hello.version != PROTOCOL_VERSION {
        tracing::warn!(
            "rejecting a client speaking protocol version {}",
            hello.version
        );
        send(
            &mut writer,
            &HelloReply::Rejected {
                version: PROTOCOL_VERSION,
            },
        )
        .await?;
        return Ok(());
    }
    send(
        &mut writer,
        &HelloReply::Accepted {
            version: PROTOCOL_VERSION,
        },
    )
    .await?;
    send(&mut writer, &context.info).await?;

    context.pane.resize(hello.cols, hello.rows);
    let mut generation = context.pane.subscribe();
    generation.borrow_and_update();
    let mut last = context.pane.screen();
    send(&mut writer, &snapshot(&last)).await?;

    let mut ended = context.ended.clone();
    loop {
        tokio::select! {
            _ = generation.changed() => {
                send_update(&mut writer, context, &mut generation, &mut last).await?;
            }
            _ = async { ended.wait_for(|ended| *ended).await.is_ok() } => {
                send_update(&mut writer, context, &mut generation, &mut last).await?;
                send(&mut writer, &ServerMessage::Exited).await?;
                return Ok(());
            }
            read = reader.read(&mut buffer) => {
                let n = read.context("cannot read from the client")?;
                if n == 0 {
                    return Ok(());
                }
                decoder.feed(&buffer[..n])?;
                while let Some(message) = decoder.next_message::<ClientMessage>()? {
                    match message {
                        ClientMessage::Key(key) => forward(context, Input::Key(key)),
                        ClientMessage::Paste(text) => forward(context, Input::Paste(text)),
                        ClientMessage::Resize { cols, rows } => context.pane.resize(cols, rows),
                        ClientMessage::Detach => return Ok(()),
                    }
                }
            }
        }
    }
}

fn forward(context: &Context, input: Input) {
    if context.input.send(input).is_err() {
        tracing::debug!("input dropped because the PTY writer has stopped");
    }
}

async fn send_update(
    writer: &mut (impl AsyncWrite + Unpin),
    context: &Context,
    generation: &mut watch::Receiver<u64>,
    last: &mut vt100::Screen,
) -> Result<()> {
    generation.borrow_and_update();
    let screen = context.pane.screen();
    let message = if screen.size() != last.size() {
        snapshot(&screen)
    } else {
        let diff = screen.state_diff(last);
        if diff.is_empty() {
            return Ok(());
        }
        ServerMessage::Update(diff)
    };
    *last = screen;
    send(writer, &message).await
}

fn snapshot(screen: &vt100::Screen) -> ServerMessage {
    let (rows, cols) = screen.size();
    ServerMessage::Snapshot {
        cols,
        rows,
        contents: screen.state_formatted(),
    }
}

async fn read_message<T: DeserializeOwned>(
    reader: &mut (impl AsyncRead + Unpin),
    decoder: &mut Decoder,
    buffer: &mut [u8],
) -> Result<Option<T>> {
    loop {
        if let Some(message) = decoder.next_message()? {
            return Ok(Some(message));
        }
        let n = reader.read(buffer).await?;
        if n == 0 {
            return Ok(None);
        }
        decoder.feed(&buffer[..n])?;
    }
}

async fn send<T: Serialize>(writer: &mut (impl AsyncWrite + Unpin), message: &T) -> Result<()> {
    writer
        .write_all(&encode(message)?)
        .await
        .context("cannot write to the client")
}
