mod callbacks;
mod connection;
mod lock;
mod pane;
mod session;

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use gband_protocol::{ExecutableId, ServerMessage, socket_path};
use portable_pty::CommandBuilder;
use tokio::net::UnixListener;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::{mpsc, watch};
use tokio::task::JoinSet;

use crate::connection::Context;
use crate::session::{Session, SessionConfig};

pub use crate::lock::{LOCK_NAME, kill, lock_path};

const SUN_PATH_MAX: usize = 107;
const FAREWELL_TIMEOUT: Duration = Duration::from_secs(1);

pub struct ServerConfig {
    pub runtime_dir: PathBuf,
    pub program: Vec<OsString>,
    pub cwd: PathBuf,
    pub executable: ExecutableId,
}

pub fn user_shell() -> OsString {
    CommandBuilder::new_default_prog().get_shell().into()
}

pub async fn run(config: ServerConfig) -> Result<()> {
    let socket = socket_path(&config.runtime_dir);
    if socket.as_os_str().len() > SUN_PATH_MAX {
        bail!(
            "socket path {} is longer than the {SUN_PATH_MAX} bytes a Unix socket allows",
            socket.display()
        );
    }
    let _lock = lock::acquire(&config.runtime_dir)?;
    let mut terminate = signal(SignalKind::terminate()).context("cannot handle SIGTERM")?;
    let (exits_tx, mut exits) = mpsc::unbounded_channel();
    let mut session = Session::start(
        SessionConfig {
            program: config.program,
            cwd: config.cwd,
            socket: socket.clone(),
        },
        exits_tx,
    )?;
    let listener = bind(&socket)?;
    tracing::info!(socket = %socket.display(), "listening");

    let (commands_tx, mut commands) = mpsc::unbounded_channel();
    let (ended_tx, ended) = watch::channel(false);
    let context = Arc::new(Context {
        state: session.state(),
        changed: session.changed(),
        commands: commands_tx,
        ended,
        info: ServerMessage::Info {
            pid: std::process::id(),
            executable: config.executable,
        },
    });
    let mut clients = JoinSet::new();
    while !session.is_over() {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    clients.spawn(connection::serve(stream, Arc::clone(&context)));
                }
                Err(error) => tracing::warn!("cannot accept a client: {error:#}"),
            },
            Some(command) = commands.recv() => session.handle(command),
            Some(exit) = exits.recv() => session.exited(exit),
            Some(()) = terminate.recv() => {
                tracing::info!("received SIGTERM, hanging up every pane");
                session.terminate();
            }
            Some(_) = clients.join_next(), if !clients.is_empty() => {}
        }
    }
    drop(listener);

    ended_tx.send_replace(true);
    let _ = tokio::time::timeout(FAREWELL_TIMEOUT, async {
        while clients.join_next().await.is_some() {}
    })
    .await;
    clients.abort_all();

    if let Err(error) = std::fs::remove_file(&socket) {
        tracing::warn!("cannot remove {}: {error:#}", socket.display());
    }
    match session.last_status() {
        Some(status) => tracing::info!("session ended, last program exited: {status}"),
        None => tracing::info!("session ended"),
    }
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
    UnixListener::bind(socket).with_context(|| format!("cannot listen on {}", socket.display()))
}
