use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::{Context as _, Result, anyhow, bail};
use gband_core::geometry::Size;
use gband_core::layout::{SessionAction, WindowId};
use gband_protocol::{
    Answer, ClientMessage, Entry, Hello, HelloReply, MessageReader, MessageWriter, Operation,
    PROTOCOL_VERSION, Process, ServerMessage, SessionName, Target, decode, leading_version,
};
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc, oneshot};
use tracing::Instrument;

use crate::channel::{Barrier, Settling};
use crate::event::SessionEvent;
use crate::hub::{Call, Hub};
use crate::registry::{Request, SessionHandle};
use crate::scripting::{self, Taps};
use crate::session::{Command, INITIAL_AREA, Reply, State};
use crate::window::{Contents, Input, Seen};

pub const ANSWER_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Context {
    pub registry: mpsc::UnboundedSender<Request>,
    pub info: ServerMessage,
    pub hub: Arc<Hub>,
    pub taps: Arc<Taps>,
}

struct Registration<'a> {
    hub: &'a Hub,
    client: u64,
}

impl Drop for Registration<'_> {
    fn drop(&mut self) {
        self.hub.detach(self.client);
        self.hub.settling.unregister(self.client);
    }
}

struct Attachment {
    handle: Arc<SessionHandle>,
    client: u64,
}

impl Attachment {
    fn new(handle: Arc<SessionHandle>, client: u64) -> Self {
        handle.clients.fetch_add(1, Ordering::AcqRel);
        handle.events.send(SessionEvent::ClientAttached { client });
        Self { handle, client }
    }
}

impl Deref for Attachment {
    type Target = SessionHandle;

    fn deref(&self) -> &SessionHandle {
        &self.handle
    }
}

impl Drop for Attachment {
    fn drop(&mut self) {
        self.handle.command(Command::Leave {
            client: self.client,
        });
        self.handle.command(Command::Shown {
            client: self.client,
            windows: Vec::new(),
        });
        self.handle.clients.fetch_sub(1, Ordering::AcqRel);
        self.handle.events.send(SessionEvent::ClientDetached {
            client: self.client,
        });
    }
}

struct Link<R, W> {
    reader: MessageReader<R>,
    writer: MessageWriter<W>,
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> Link<R, W> {
    async fn send<T: Serialize>(&mut self, message: &T) -> Result<()> {
        send(&mut self.writer, message).await
    }
}

#[derive(Default)]
struct Sent {
    state: Option<Arc<State>>,
    windows: HashMap<WindowId, (u64, Seen)>,
    names: HashMap<WindowId, u64>,
}

pub async fn serve(
    reader: impl AsyncRead + Unpin,
    writer: impl AsyncWrite + Unpin,
    context: Arc<Context>,
) {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    tracing::debug!(client = id, "client connected");
    let link = Link {
        reader: MessageReader::new(reader),
        writer: MessageWriter::new(writer),
    };
    match handle(link, &context, id).await {
        Ok(()) => tracing::debug!(client = id, "client disconnected"),
        Err(error) => tracing::warn!(client = id, "closing client connection: {error:#}"),
    }
}

async fn handle(
    mut link: Link<impl AsyncRead + Unpin, impl AsyncWrite + Unpin>,
    context: &Context,
    client: u64,
) -> Result<()> {
    let Some(payload) = link.reader.recv_payload().await.context("invalid hello")? else {
        return Ok(());
    };
    let Some(version) = leading_version(&payload) else {
        bail!("the first frame does not begin with a protocol version");
    };
    if version != PROTOCOL_VERSION {
        tracing::warn!("rejecting a client speaking protocol version {version}");
        link.send(&HelloReply::Rejected {
            version: PROTOCOL_VERSION,
        })
        .await?;
        return Ok(());
    }
    let hello: Hello = decode(&payload).context("invalid hello")?;
    link.send(&HelloReply::Accepted {
        version: PROTOCOL_VERSION,
    })
    .await?;
    link.send(&context.info).await?;

    let Some(request) = link
        .reader
        .recv::<ClientMessage>()
        .await
        .context("invalid request")?
    else {
        return Ok(());
    };
    match request {
        ClientMessage::Attach { session, cwd } => {
            let size = Size::new(hello.cols, hello.rows);
            attach(link, context, client, session, cwd, size).await
        }
        ClientMessage::ListSessions => {
            let sessions = ask(context, |reply| Request::List { reply }).await?;
            link.send(&ServerMessage::Sessions(sessions)).await
        }
        ClientMessage::KillSession { session } => {
            let handle = ask(context, |reply| Request::Kill {
                name: session,
                reply,
            })
            .await?;
            let Some(handle) = handle else {
                return link.send(&ServerMessage::NoSuchSession).await;
            };
            let mut ended = handle.ended.clone();
            let _ = ended.wait_for(|ended| *ended).await;
            link.send(&ServerMessage::Killed).await
        }
        ClientMessage::Control {
            session,
            target,
            operation,
        } => {
            let answer = control(context, session, target, operation).await;
            link.send(&answer).await
        }
        _ => bail!("the first message after info is not a request"),
    }
}

async fn control(
    context: &Context,
    session: SessionName,
    target: Target,
    operation: Operation,
) -> ServerMessage {
    if context.hub.session(session.as_str()).is_none() {
        return ServerMessage::NoSuchSession;
    }
    let mut entries = Vec::new();
    if matches!(target, Target::Server | Target::All) {
        let answer = server_part(context, &session, &operation).await;
        entries.push(Entry {
            process: Process::Server,
            answer: Some(answer),
        });
    }
    let attached = context.hub.clients_of(&session);
    let clients = match target {
        Target::Server => Vec::new(),
        Target::EveryClient | Target::All => attached,
        Target::Client(client) => attached.into_iter().filter(|&id| id == client).collect(),
        Target::Chosen => context.hub.chosen(&session).into_iter().collect(),
    };
    let calls: Vec<(u64, Option<Call>)> = clients
        .into_iter()
        .map(|client| (client, context.hub.control(client, operation.clone())))
        .collect();
    let deadline = tokio::time::Instant::now() + ANSWER_TIMEOUT;
    for (client, call) in calls {
        let answer = match call {
            Some((call, answer)) => {
                let answered = tokio::time::timeout_at(deadline, answer).await;
                context.hub.forget(call);
                answered.ok().and_then(Result::ok)
            }
            None => None,
        };
        entries.push(Entry {
            process: Process::Client(client),
            answer,
        });
    }
    ServerMessage::ControlResults(entries)
}

async fn server_part(context: &Context, session: &SessionName, operation: &Operation) -> Answer {
    let no_lua = || "the server runs no Lua".to_owned();
    match operation.clone() {
        Operation::Reload => match context.taps.load().await {
            Some((load, error)) => Answer::Loaded { load, error },
            None => Answer::Loaded {
                load: 1,
                error: Some(no_lua()),
            },
        },
        Operation::Errors => {
            let (load, errors) = context.taps.errors().await.unwrap_or((1, Vec::new()));
            Answer::Errors { load, errors }
        }
        Operation::Eval { source, args } => Answer::Values(context.taps.eval(source, args).await),
        Operation::Command { name, args } => {
            let client = context.hub.chosen(session);
            let (replies_tx, mut replies) = mpsc::unbounded_channel();
            let called = context.taps.call(scripting::Input::Call {
                session: session.clone(),
                client: client.unwrap_or(scripting::LUA_CLIENT),
                call: 0,
                name: name.clone(),
                args,
                replies: replies_tx,
            });
            if !called {
                return Answer::Values(Err(format!("unknown command `{name}`")));
            }
            while let Some(reply) = replies.recv().await {
                match reply {
                    Reply::Result { result, .. } => {
                        return Answer::Values(result.map(|value| vec![value]));
                    }
                    Reply::Focus(window) => {
                        if let Some(client) = client {
                            context.hub.send(client, ServerMessage::Focus(window));
                        }
                    }
                    Reply::Opened { .. } => {}
                }
            }
            Answer::Values(Err(no_lua()))
        }
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
    mut link: Link<impl AsyncRead + Unpin, impl AsyncWrite + Unpin>,
    context: &Context,
    client: u64,
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
    let session = Attachment::new(handle, client);
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
    sync(&mut link.writer, &session, &mut sent, &context.hub.settling).await?;

    let (deliveries_tx, mut deliveries) = mpsc::unbounded_channel();
    let attached = context.hub.attach(client, &session.name, deliveries_tx);
    let (barriers_tx, mut barriers) = mpsc::unbounded_channel();
    context.hub.settling.register(client, barriers_tx);
    let _registration = Registration {
        hub: &context.hub,
        client,
    };
    let settling = &context.hub.settling;
    for message in &attached.messages {
        deliver(&mut link.writer, message, settling).await?;
    }

    let (reply_tx, mut replies) = mpsc::unbounded_channel();
    let mut ended = session.ended.clone();
    loop {
        if dispatch(&mut link.reader, &session, client, &reply_tx, context)? {
            return Ok(());
        }
        tokio::select! {
            _ = changed.changed() => {
                changed.borrow_and_update();
                sync(&mut link.writer, &session, &mut sent, settling).await?;
            }
            Some(reply) = replies.recv() => {
                deliver_reply(&mut link.writer, reply, &session, &mut sent, settling).await?;
            }
            Some(message) = deliveries.recv() => {
                sync(&mut link.writer, &session, &mut sent, settling).await?;
                deliver(&mut link.writer, &message, settling).await?;
            }
            Some(barrier) = barriers.recv() => match barrier {
                Barrier::Read(reached) => {
                    if !link.reader.fill_ready().await.context("cannot read from the client")? {
                        return Ok(());
                    }
                    if dispatch(&mut link.reader, &session, client, &reply_tx, context)? {
                        return Ok(());
                    }
                    session.command(Command::Barrier(reached));
                }
                Barrier::Flush(reached) => {
                    changed.borrow_and_update();
                    sync(&mut link.writer, &session, &mut sent, settling).await?;
                    while let Ok(reply) = replies.try_recv() {
                        deliver_reply(&mut link.writer, reply, &session, &mut sent, settling).await?;
                    }
                    while let Ok(message) = deliveries.try_recv() {
                        deliver(&mut link.writer, &message, settling).await?;
                    }
                    changed.borrow_and_update();
                    sync(&mut link.writer, &session, &mut sent, settling).await?;
                    let _ = reached.send(());
                }
            },
            _ = async { ended.wait_for(|ended| *ended).await.is_ok() } => {
                sync(&mut link.writer, &session, &mut sent, settling).await?;
                link.send(&ServerMessage::Exited).await?;
                return Ok(());
            }
            filled = link.reader.fill() => {
                if !filled.context("cannot read from the client")? {
                    return Ok(());
                }
            }
        }
    }
}

fn dispatch(
    reader: &mut MessageReader<impl AsyncRead + Unpin>,
    session: &SessionHandle,
    client: u64,
    reply_tx: &mpsc::UnboundedSender<Reply>,
    context: &Context,
) -> Result<bool> {
    while let Some(message) = reader.try_recv::<ClientMessage>()? {
        match message {
            ClientMessage::Key { window, key } => {
                context.hub.typed(client);
                forward(session, window, Input::Key(key), || {
                    context.taps.notice(&session.name, window, client);
                });
            }
            ClientMessage::Paste { window, text } => {
                context.hub.typed(client);
                forward(session, window, Input::Paste(text), || {
                    context.taps.notice(&session.name, window, client);
                });
            }
            ClientMessage::Mouse { window, event } => {
                context.hub.typed(client);
                forward(session, window, Input::Mouse(event), || {});
            }
            ClientMessage::Command { call, name, args } => {
                let call_input = scripting::Input::Call {
                    session: session.name.clone(),
                    client,
                    call,
                    name: name.clone(),
                    args,
                    replies: reply_tx.clone(),
                };
                if !context.taps.call(call_input) {
                    let _ = reply_tx.send(Reply::Result {
                        call,
                        result: Err(format!("unknown command `{name}`")),
                    });
                }
            }
            ClientMessage::Resize { cols, rows } => session.command(Command::Area {
                size: Size::new(cols, rows),
                applied: None,
            }),
            ClientMessage::Action(action) => {
                let reply =
                    matches!(action, SessionAction::OpenWindow { .. }).then(|| reply_tx.clone());
                session.command(Command::Action {
                    client,
                    action,
                    reply,
                });
            }
            ClientMessage::Content { window, output } => session.command(Command::Content {
                client,
                window,
                output,
            }),
            ClientMessage::Shown(windows) => session.command(Command::Shown { client, windows }),
            ClientMessage::Rename { window, name } => {
                session.command(Command::Rename { window, name });
            }
            ClientMessage::Reload => {
                let taps = Arc::clone(&context.taps);
                let hub = Arc::clone(&context.hub);
                tokio::spawn(
                    async move {
                        taps.load().await;
                        hub.send(client, ServerMessage::Reloaded);
                    }
                    .in_current_span(),
                );
            }
            ClientMessage::ControlAnswer { call, answer } => context.hub.answer(call, answer),
            ClientMessage::Detach => return Ok(true),
            ClientMessage::Attach { .. }
            | ClientMessage::ListSessions
            | ClientMessage::KillSession { .. }
            | ClientMessage::Control { .. } => bail!("a request after the client attached"),
        }
    }
    Ok(false)
}

fn replied(reply: Reply) -> ServerMessage {
    match reply {
        Reply::Focus(window) => ServerMessage::Focus(window),
        Reply::Opened { request, window } => ServerMessage::Opened { request, window },
        Reply::Result { call, result } => ServerMessage::Result { call, result },
    }
}

async fn deliver_reply(
    writer: &mut MessageWriter<impl AsyncWrite + Unpin>,
    reply: Reply,
    session: &SessionHandle,
    sent: &mut Sent,
    settling: &Settling,
) -> Result<()> {
    sync(writer, session, sent, settling).await?;
    if let Reply::Focus(window) = reply
        && !sent
            .state
            .as_ref()
            .is_some_and(|state| state.layout.contains(window))
    {
        tracing::debug!(window = %window, "focus dropped for a window no longer in the layout");
        return Ok(());
    }
    deliver(writer, &replied(reply), settling).await
}

async fn deliver(
    writer: &mut MessageWriter<impl AsyncWrite + Unpin>,
    message: &ServerMessage,
    settling: &Settling,
) -> Result<()> {
    settling.sent(message);
    send(writer, message).await
}

fn forward(
    session: &SessionHandle,
    window: WindowId,
    input: Input,
    before_send: impl FnOnce(),
) -> bool {
    let state = session.state.borrow();
    let Some(entry) = state.windows.get(&window) else {
        tracing::debug!(window = %window, "input dropped for a window not in the layout");
        return false;
    };
    before_send();
    if entry.input.send(input).is_err() {
        tracing::debug!(window = %window, "input dropped because the PTY writer has stopped");
        return false;
    }
    true
}

async fn sync(
    writer: &mut MessageWriter<impl AsyncWrite + Unpin>,
    session: &SessionHandle,
    sent: &mut Sent,
    settling: &Settling,
) -> Result<()> {
    let state = Arc::clone(&session.state.borrow());
    if !sent
        .state
        .as_ref()
        .is_some_and(|last| Arc::ptr_eq(last, &state))
    {
        deliver(
            writer,
            &ServerMessage::Layout {
                cols: state.area.cols,
                rows: state.area.rows,
                layout: state.layout.clone(),
            },
            settling,
        )
        .await?;
        sent.windows
            .retain(|window, _| state.windows.contains_key(window));
        sent.names
            .retain(|window, _| state.windows.contains_key(window));
        sent.state = Some(Arc::clone(&state));
    }
    for window in state.layout.windows() {
        let Some(entry) = state.windows.get(&window) else {
            continue;
        };
        let generation = entry.window.generation();
        if sent
            .windows
            .get(&window)
            .is_some_and(|(last, _)| *last == generation)
        {
            continue;
        }
        let (generation, seen, contents) = entry
            .window
            .catch_up(sent.windows.get(&window).map(|(_, seen)| seen));
        sent.windows.insert(window, (generation, seen));
        let message = match contents {
            Contents::Update(contents) if contents.is_empty() => continue,
            Contents::Update(contents) => ServerMessage::Update { window, contents },
            Contents::Snapshot(size, contents) => ServerMessage::Snapshot {
                window,
                cols: size.cols,
                rows: size.rows,
                contents,
            },
        };
        deliver(writer, &message, settling).await?;
    }
    for window in state.layout.windows() {
        let Some(names) = state
            .windows
            .get(&window)
            .and_then(|entry| entry.window.names())
        else {
            continue;
        };
        if sent.names.get(&window) == Some(&names.generation) {
            continue;
        }
        sent.names.insert(window, names.generation);
        let message = ServerMessage::WindowName {
            window,
            automatic: names.automatic,
            manual: names.manual,
        };
        deliver(writer, &message, settling).await?;
    }
    Ok(())
}

async fn send<T: Serialize>(
    writer: &mut MessageWriter<impl AsyncWrite + Unpin>,
    message: &T,
) -> Result<()> {
    writer
        .send(message)
        .await
        .context("cannot write to the client")
}
