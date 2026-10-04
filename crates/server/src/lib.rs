mod callbacks;
mod connection;
mod lock;
mod pane;

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use gband_protocol::{ExecutableId, ServerMessage, socket_path};
use portable_pty::CommandBuilder;
use rustix::process::{Pid, Signal};
use tokio::net::UnixListener;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::watch;
use tokio::task::JoinSet;

use crate::connection::Context;
use crate::pane::{Pane, Session};

pub use crate::lock::{LOCK_NAME, kill, lock_path};

const SUN_PATH_MAX: usize = 107;
const KILL_GRACE: Duration = Duration::from_secs(2);
const DRAIN_TIMEOUT: Duration = Duration::from_millis(500);
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
    let Session {
        pane,
        input,
        mut exit,
        mut drained,
        mut killer,
        pid,
    } = pane::spawn(&config.program, &config.cwd, &socket)?;
    let listener = bind(&socket)?;
    tracing::info!(socket = %socket.display(), "listening");

    let (ended_tx, ended) = watch::channel(false);
    let context = Arc::new(Context {
        pane: Arc::clone(&pane),
        input,
        ended,
        info: ServerMessage::Info {
            pid: std::process::id(),
            executable: config.executable,
        },
    });
    let mut clients = JoinSet::new();
    loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    clients.spawn(connection::serve(stream, Arc::clone(&context)));
                }
                Err(error) => tracing::warn!("cannot accept a client: {error:#}"),
            },
            _ = async { exit.wait_for(Option::is_some).await.is_ok() } => break,
            Some(()) = terminate.recv() => {
                tracing::info!("received SIGTERM, hanging up the program");
                if let Err(error) = killer.kill() {
                    tracing::warn!("cannot send SIGHUP to the program: {error:#}");
                }
                tokio::spawn(kill_if_running(exit.clone(), Arc::clone(&pane), pid));
            }
            Some(_) = clients.join_next(), if !clients.is_empty() => {}
        }
    }
    drop(listener);

    let _ = tokio::time::timeout(DRAIN_TIMEOUT, drained.wait_for(|drained| *drained)).await;
    ended_tx.send_replace(true);
    let _ = tokio::time::timeout(FAREWELL_TIMEOUT, async {
        while clients.join_next().await.is_some() {}
    })
    .await;
    clients.abort_all();

    if let Err(error) = std::fs::remove_file(&socket) {
        tracing::warn!("cannot remove {}: {error:#}", socket.display());
    }
    match exit.borrow().as_ref() {
        Some(status) => tracing::info!("program exited: {status}"),
        None => tracing::info!("program exited"),
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

async fn kill_if_running(
    mut exit: watch::Receiver<Option<portable_pty::ExitStatus>>,
    pane: Arc<Pane>,
    pid: Option<u32>,
) {
    let exited = tokio::time::timeout(KILL_GRACE, exit.wait_for(Option::is_some)).await;
    if exited.is_ok() {
        return;
    }
    tracing::warn!("program still running after SIGHUP, sending SIGKILL");
    if let Some(group) = pane.foreground_group().and_then(Pid::from_raw) {
        let _ = rustix::process::kill_process_group(group, Signal::KILL);
    }
    if let Some(program) = pid.and_then(|pid| Pid::from_raw(pid as i32)) {
        let _ = rustix::process::kill_process(program, Signal::KILL);
    }
}
