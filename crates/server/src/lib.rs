mod channel;
mod connection;
mod event;
mod hub;
mod lock;
mod registry;
mod scripting;
mod session;
mod window;

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use gband_core::geometry::Size;
use gband_core::layout::LayoutOptions;
use gband_protocol::{ExecutableId, ServerMessage, SessionName};
use portable_pty::CommandBuilder;
use rustix::fs::Mode;
use tokio::net::UnixListener;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio::task::JoinSet;
use tracing::Instrument;

pub use crate::channel::{Loader, TestChannel};
use crate::connection::Context;
pub use crate::event::{CAPACITY, Published, SessionEvent};
use crate::hub::Hub;
pub use crate::hub::QUEUE_LIMIT;
pub use crate::lock::kill;
use crate::registry::{Registry, Shared};
use crate::scripting::Taps;
pub use crate::scripting::{NOTICE_BUDGET, OUTPUT_BUDGET, Reloader, Scripting};
pub use crate::session::INITIAL_AREA;

pub const SUN_PATH_MAX: usize = 107;
const PRIVATE_SOCKET_MASK: u32 = 0o177;
const FAREWELL_TIMEOUT: Duration = Duration::from_secs(1);

pub struct ServerConfig {
    pub socket: PathBuf,
    pub session: SessionName,
    pub program: Vec<OsString>,
    pub cwd: PathBuf,
    pub area: Size,
    pub executable: ExecutableId,
    pub options: watch::Receiver<LayoutOptions>,
    pub scripting: Option<Scripting>,
    pub channel: Option<TestChannel>,
}

pub fn user_shell() -> OsString {
    CommandBuilder::new_default_prog().get_shell().into()
}

pub async fn run(config: ServerConfig) -> Result<()> {
    run_with_events(config, event::channel()).await
}

pub async fn run_with_events(
    config: ServerConfig,
    events: broadcast::Sender<Published>,
) -> Result<()> {
    let socket = config.socket;
    if socket.as_os_str().len() > SUN_PATH_MAX {
        bail!(
            "socket path {} is longer than the {SUN_PATH_MAX} bytes a Unix socket allows",
            socket.display()
        );
    }
    let _lock = lock::acquire(&socket)?;
    let mut terminate = signal(SignalKind::terminate()).context("cannot handle SIGTERM")?;
    tokio::spawn(event::log(events.subscribe()).in_current_span());
    let hub = Arc::new(Hub::default());
    let taps = match config.scripting {
        Some(scripting) => scripting::start(scripting, Arc::clone(&hub), &events),
        None => Taps::idle(),
    };
    let mut closed = match config.channel {
        Some(channel) => {
            let (closed_tx, closed) = oneshot::channel();
            let serving = channel::serve(channel, Arc::clone(&hub), Arc::clone(&taps));
            tokio::spawn(
                async move {
                    serving.await;
                    let _ = closed_tx.send(());
                }
                .in_current_span(),
            );
            Some(closed)
        }
        None => None,
    };
    let (requests_tx, mut requests) = mpsc::unbounded_channel();
    let mut registry = Registry::new(
        config.program,
        socket.clone(),
        requests_tx.clone(),
        Shared {
            events,
            options: config.options,
            hub: Arc::clone(&hub),
            taps: Arc::clone(&taps),
        },
    );
    registry.create(config.session, config.cwd, config.area)?;
    let listener = bind(&socket)?;
    tracing::info!("listening");

    let context = Arc::new(Context {
        registry: requests_tx,
        info: ServerMessage::Info {
            pid: std::process::id(),
            executable: config.executable,
        },
        hub,
        taps,
    });
    let mut clients = JoinSet::new();
    while !registry.is_empty() {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    let (reader, writer) = stream.into_split();
                    clients.spawn(
                        connection::serve(reader, writer, Arc::clone(&context)).in_current_span(),
                    );
                }
                Err(error) => tracing::warn!("cannot accept a client: {error:#}"),
            },
            Some(request) = requests.recv() => registry.handle(request),
            Some(()) = terminate.recv() => {
                tracing::info!("received SIGTERM, hanging up every window of every session");
                registry.terminate();
            }
            _ = async { closed.as_mut().expect("guarded by is_some").await }, if closed.is_some() => {
                closed = None;
                tracing::info!("the test channel closed, hanging up every window of every session");
                registry.terminate();
            }
            Some(_) = clients.join_next(), if !clients.is_empty() => {}
        }
    }
    drop(listener);
    drop(requests);

    let _ = tokio::time::timeout(FAREWELL_TIMEOUT, async {
        while clients.join_next().await.is_some() {}
    })
    .await;
    clients.abort_all();

    if let Err(error) = std::fs::remove_file(&socket) {
        tracing::warn!("cannot remove {}: {error:#}", socket.display());
    }
    tracing::info!("last session ended");
    Ok(())
}

fn bind(socket: &Path) -> Result<UnixListener> {
    match std::fs::remove_file(socket) {
        Ok(()) => tracing::info!("replaced a stale socket at {}", socket.display()),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("cannot remove {}", socket.display()));
        }
    }
    let previous = rustix::process::umask(Mode::from_raw_mode(PRIVATE_SOCKET_MASK));
    let listener = UnixListener::bind(socket);
    rustix::process::umask(previous);
    listener.with_context(|| format!("cannot listen on {}", socket.display()))
}
