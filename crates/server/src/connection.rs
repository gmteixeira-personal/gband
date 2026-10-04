use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context as _, Result};
use gband_core::geometry::Size;
use gband_core::layout::{PaneId, SessionAction};
use gband_protocol::{
    ClientMessage, Decoder, Hello, HelloReply, PROTOCOL_VERSION, ServerMessage, encode,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::{mpsc, oneshot, watch};

use crate::pane::Input;
use crate::session::{Command, State};

const READ_BUFFER_LEN: usize = 64 * 1024;

pub struct Context {
    pub state: watch::Receiver<Arc<State>>,
    pub changed: watch::Receiver<u64>,
    pub commands: mpsc::UnboundedSender<Command>,
    pub ended: watch::Receiver<bool>,
    pub info: ServerMessage,
}

#[derive(Default)]
struct Sent {
    state: Option<Arc<State>>,
    panes: HashMap<PaneId, (u64, vt100::Screen)>,
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

    let (applied_tx, applied) = oneshot::channel();
    command(
        context,
        Command::Area {
            size: Size::new(hello.cols, hello.rows),
            applied: Some(applied_tx),
        },
    );
    let _ = applied.await;

    let mut changed = context.changed.clone();
    changed.borrow_and_update();
    let mut sent = Sent::default();
    sync(&mut writer, context, &mut sent).await?;

    let (focus_tx, mut focus) = mpsc::unbounded_channel();
    let mut ended = context.ended.clone();
    loop {
        tokio::select! {
            _ = changed.changed() => {
                changed.borrow_and_update();
                sync(&mut writer, context, &mut sent).await?;
            }
            Some(pane) = focus.recv() => {
                sync(&mut writer, context, &mut sent).await?;
                send(&mut writer, &ServerMessage::Focus(pane)).await?;
            }
            _ = async { ended.wait_for(|ended| *ended).await.is_ok() } => {
                sync(&mut writer, context, &mut sent).await?;
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
                        ClientMessage::Key { pane, key } => forward(context, pane, Input::Key(key)),
                        ClientMessage::Paste { pane, text } => {
                            forward(context, pane, Input::Paste(text));
                        }
                        ClientMessage::Resize { cols, rows } => command(
                            context,
                            Command::Area {
                                size: Size::new(cols, rows),
                                applied: None,
                            },
                        ),
                        ClientMessage::Action(action) => {
                            let focus = matches!(action, SessionAction::OpenPane { .. })
                                .then(|| focus_tx.clone());
                            command(context, Command::Action { action, focus });
                        }
                        ClientMessage::Detach => return Ok(()),
                    }
                }
            }
        }
    }
}

fn command(context: &Context, command: Command) {
    if context.commands.send(command).is_err() {
        tracing::debug!("command dropped because the session has ended");
    }
}

fn forward(context: &Context, pane: PaneId, input: Input) {
    let state = context.state.borrow();
    let Some(entry) = state.panes.get(&pane) else {
        tracing::debug!(pane = %pane, "input dropped for a pane not in the layout");
        return;
    };
    if entry.input.send(input).is_err() {
        tracing::debug!(pane = %pane, "input dropped because the PTY writer has stopped");
    }
}

async fn sync(
    writer: &mut (impl AsyncWrite + Unpin),
    context: &Context,
    sent: &mut Sent,
) -> Result<()> {
    let state = Arc::clone(&context.state.borrow());
    if !sent
        .state
        .as_ref()
        .is_some_and(|last| Arc::ptr_eq(last, &state))
    {
        send(
            writer,
            &ServerMessage::Layout {
                cols: state.area.cols,
                rows: state.area.rows,
                layout: state.layout.clone(),
            },
        )
        .await?;
        sent.panes.retain(|pane, _| state.panes.contains_key(pane));
        sent.state = Some(Arc::clone(&state));
    }
    for pane in state.layout.panes() {
        let Some(entry) = state.panes.get(&pane) else {
            continue;
        };
        let generation = entry.pane.generation();
        if sent
            .panes
            .get(&pane)
            .is_some_and(|(last, _)| *last == generation)
        {
            continue;
        }
        let (generation, screen) = entry.pane.screen();
        let message = match sent.panes.get(&pane) {
            Some((_, last)) if last.size() == screen.size() => {
                let contents = screen.state_diff(last);
                if contents.is_empty() {
                    None
                } else {
                    Some(ServerMessage::Update { pane, contents })
                }
            }
            _ => Some(snapshot(pane, &screen)),
        };
        sent.panes.insert(pane, (generation, screen));
        if let Some(message) = message {
            send(writer, &message).await?;
        }
    }
    Ok(())
}

fn snapshot(pane: PaneId, screen: &vt100::Screen) -> ServerMessage {
    let (rows, cols) = screen.size();
    ServerMessage::Snapshot {
        pane,
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
