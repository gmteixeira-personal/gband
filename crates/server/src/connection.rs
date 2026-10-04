use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context as _, Result, anyhow, bail};
use gband_core::geometry::Size;
use gband_core::layout::{PaneId, SessionAction};
use gband_protocol::{
    ClientMessage, Decoder, Hello, HelloReply, PROTOCOL_VERSION, ServerMessage, SessionName, encode,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{mpsc, oneshot};

use crate::pane::Input;
use crate::registry::{Request, SessionHandle};
use crate::session::{Command, INITIAL_AREA, State};

const READ_BUFFER_LEN: usize = 64 * 1024;

pub struct Context {
    pub registry: mpsc::UnboundedSender<Request>,
    pub info: ServerMessage,
}

struct Attachment(Arc<SessionHandle>);

impl Attachment {
    fn new(handle: Arc<SessionHandle>) -> Self {
        handle.clients.fetch_add(1, Ordering::AcqRel);
        Self(handle)
    }
}

impl Deref for Attachment {
    type Target = SessionHandle;

    fn deref(&self) -> &SessionHandle {
        &self.0
    }
}

impl Drop for Attachment {
    fn drop(&mut self) {
        self.0.clients.fetch_sub(1, Ordering::AcqRel);
    }
}

struct Link {
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf,
    decoder: Decoder,
    buffer: Vec<u8>,
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
    let (reader, writer) = stream.into_split();
    let mut link = Link {
        reader,
        writer,
        decoder: Decoder::new(),
        buffer: vec![0; READ_BUFFER_LEN],
    };

    let Some(hello) = link.read::<Hello>().await.context("invalid hello")? else {
        return Ok(());
    };
    if hello.version != PROTOCOL_VERSION {
        tracing::warn!(
            "rejecting a client speaking protocol version {}",
            hello.version
        );
        send(
            &mut link.writer,
            &HelloReply::Rejected {
                version: PROTOCOL_VERSION,
            },
        )
        .await?;
        return Ok(());
    }
    send(
        &mut link.writer,
        &HelloReply::Accepted {
            version: PROTOCOL_VERSION,
        },
    )
    .await?;
    send(&mut link.writer, &context.info).await?;

    let Some(request) = link
        .read::<ClientMessage>()
        .await
        .context("invalid request")?
    else {
        return Ok(());
    };
    match request {
        ClientMessage::Attach { session, cwd } => {
            let size = Size::new(hello.cols, hello.rows);
            attach(link, context, session, cwd, size).await
        }
        ClientMessage::ListSessions => {
            let sessions = ask(context, |reply| Request::List { reply }).await?;
            send(&mut link.writer, &ServerMessage::Sessions(sessions)).await
        }
        ClientMessage::KillSession { session } => {
            let handle = ask(context, |reply| Request::Kill {
                name: session,
                reply,
            })
            .await?;
            let Some(handle) = handle else {
                return send(&mut link.writer, &ServerMessage::NoSuchSession).await;
            };
            let mut ended = handle.ended.clone();
            let _ = ended.wait_for(|ended| *ended).await;
            send(&mut link.writer, &ServerMessage::Killed).await
        }
        _ => bail!("the first message after info is not a request"),
    }
}

async fn ask<T>(
    context: &Context,
    request: impl FnOnce(oneshot::Sender<T>) -> Request,
) -> Result<T> {
    let (reply, answer) = oneshot::channel();
    context
        .registry
        .send(request(reply))
        .map_err(|_| anyhow!("the server is shutting down"))?;
    answer.await.context("the server is shutting down")
}

async fn attach(
    mut link: Link,
    context: &Context,
    name: SessionName,
    cwd: PathBuf,
    size: Size,
) -> Result<()> {
    let area = if size.cols > 0 && size.rows > 0 {
        size
    } else {
        INITIAL_AREA
    };
    let handle = ask(context, |reply| Request::Attach {
        name,
        cwd,
        area,
        reply,
    })
    .await??;
    let session = Attachment::new(handle);
    tracing::debug!(session = %session.name, "client attached");

    let (applied_tx, applied) = oneshot::channel();
    session.command(Command::Area {
        size,
        applied: Some(applied_tx),
    });
    let _ = applied.await;

    let mut changed = session.changed.clone();
    changed.borrow_and_update();
    let mut sent = Sent::default();
    sync(&mut link.writer, &session, &mut sent).await?;

    let (focus_tx, mut focus) = mpsc::unbounded_channel();
    let mut ended = session.ended.clone();
    loop {
        if dispatch(&mut link.decoder, &session, &focus_tx)? {
            return Ok(());
        }
        tokio::select! {
            _ = changed.changed() => {
                changed.borrow_and_update();
                sync(&mut link.writer, &session, &mut sent).await?;
            }
            Some(pane) = focus.recv() => {
                sync(&mut link.writer, &session, &mut sent).await?;
                send(&mut link.writer, &ServerMessage::Focus(pane)).await?;
            }
            _ = async { ended.wait_for(|ended| *ended).await.is_ok() } => {
                sync(&mut link.writer, &session, &mut sent).await?;
                send(&mut link.writer, &ServerMessage::Exited).await?;
                return Ok(());
            }
            read = link.reader.read(&mut link.buffer) => {
                let n = read.context("cannot read from the client")?;
                if n == 0 {
                    return Ok(());
                }
                link.decoder.feed(&link.buffer[..n])?;
            }
        }
    }
}

fn dispatch(
    decoder: &mut Decoder,
    session: &SessionHandle,
    focus_tx: &mpsc::UnboundedSender<PaneId>,
) -> Result<bool> {
    while let Some(message) = decoder.next_message::<ClientMessage>()? {
        match message {
            ClientMessage::Key { pane, key } => forward(session, pane, Input::Key(key)),
            ClientMessage::Paste { pane, text } => forward(session, pane, Input::Paste(text)),
            ClientMessage::Resize { cols, rows } => session.command(Command::Area {
                size: Size::new(cols, rows),
                applied: None,
            }),
            ClientMessage::Action(action) => {
                let focus =
                    matches!(action, SessionAction::OpenPane { .. }).then(|| focus_tx.clone());
                session.command(Command::Action { action, focus });
            }
            ClientMessage::Detach => return Ok(true),
            ClientMessage::Attach { .. }
            | ClientMessage::ListSessions
            | ClientMessage::KillSession { .. } => bail!("a request after the client attached"),
        }
    }
    Ok(false)
}

fn forward(session: &SessionHandle, pane: PaneId, input: Input) {
    let state = session.state.borrow();
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
    session: &SessionHandle,
    sent: &mut Sent,
) -> Result<()> {
    let state = Arc::clone(&session.state.borrow());
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

impl Link {
    async fn read<T: DeserializeOwned>(&mut self) -> Result<Option<T>> {
        loop {
            if let Some(message) = self.decoder.next_message()? {
                return Ok(Some(message));
            }
            let n = self.reader.read(&mut self.buffer).await?;
            if n == 0 {
                return Ok(None);
            }
            self.decoder.feed(&self.buffer[..n])?;
        }
    }
}

async fn send<T: Serialize>(writer: &mut (impl AsyncWrite + Unpin), message: &T) -> Result<()> {
    writer
        .write_all(&encode(message)?)
        .await
        .context("cannot write to the client")
}
