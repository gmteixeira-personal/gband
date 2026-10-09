mod channel;
mod connection;
mod event;
mod hub;
mod lock;
mod process;
mod registry;
mod scripting;
mod session;
mod window;

use std::ffi::OsString;
use std::fs::DirBuilder;
use std::io::ErrorKind;
use std::os::fd::AsRawFd;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use gband_core::geometry::Size;
use gband_core::layout::LayoutOptions;
use gband_protocol::{ExecutableId, ServerMessage, SessionName};
use portable_pty::CommandBuilder;
use rustix::fs::{AtFlags, Mode, OFlags};
use tokio::net::UnixListener;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio::task::JoinSet;
use tracing::Instrument;

pub use crate::channel::TestChannel;
pub use crate::connection::ANSWER_TIMEOUT;
use crate::connection::Context;
pub use crate::event::{CAPACITY, Published, SessionEvent};
use crate::hub::Hub;
pub use crate::hub::QUEUE_LIMIT;
pub use crate::lock::kill;
use crate::registry::{Registry, Shared};
use crate::scripting::Taps;
pub use crate::scripting::{Loader, NOTICE_BUDGET, OUTPUT_BUDGET, Reloader, Scripting};
pub use crate::session::INITIAL_AREA;

pub const SUN_PATH_MAX: usize = 107;
const PRIVATE_DIRECTORY_PREFIX: &str = ".gband-bind-";
const PRIVATE_DIRECTORY_ATTEMPTS: u32 = 100;
const BOUND_NAME: &str = "s";
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
    let parent = socket
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let directory = private_directory(parent)?;
    let listener = bind_within(&directory, socket);
    let _ = std::fs::remove_file(directory.join(BOUND_NAME));
    let _ = std::fs::remove_dir(&directory);
    listener.with_context(|| format!("cannot listen on {}", socket.display()))
}

fn private_directory(parent: &Path) -> Result<PathBuf> {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    for _ in 0..PRIVATE_DIRECTORY_ATTEMPTS {
        let directory = parent.join(format!(
            "{PRIVATE_DIRECTORY_PREFIX}{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match DirBuilder::new().mode(0o700).create(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("cannot create {}", directory.display()));
            }
        }
    }
    bail!(
        "cannot create a private directory in {}: every name was taken",
        parent.display()
    )
}

fn bind_within(directory: &Path, socket: &Path) -> std::io::Result<UnixListener> {
    let held = rustix::fs::open(
        directory,
        OFlags::PATH | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )?;
    let listener = UnixListener::bind(format!("/proc/self/fd/{}/{BOUND_NAME}", held.as_raw_fd()))?;
    rustix::fs::chmodat(
        &held,
        BOUND_NAME,
        Mode::from_raw_mode(0o600),
        AtFlags::empty(),
    )?;
    rustix::fs::renameat(&held, BOUND_NAME, rustix::fs::CWD, socket)?;
    Ok(listener)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Barrier;

    use super::*;

    struct Scratch(gband_scratch::Scratch);

    impl Scratch {
        fn new(name: &str) -> Self {
            Self(gband_scratch::Scratch::new("bind", name))
        }

        fn leftovers(&self) -> Vec<String> {
            fs::read_dir(&self.0)
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .filter(|name| name.starts_with(PRIVATE_DIRECTORY_PREFIX))
                .collect()
        }
    }

    fn umask() -> String {
        fs::read_to_string("/proc/self/status")
            .unwrap()
            .lines()
            .find_map(|line| line.strip_prefix("Umask:"))
            .unwrap()
            .trim()
            .to_owned()
    }

    fn mode(path: &Path) -> u32 {
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn concurrent_binds_are_private_and_keep_the_umask() {
        const BINDS: usize = 8;
        let scratch = Scratch::new("concurrent");
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .unwrap();
        let before = umask();
        let barrier = Barrier::new(BINDS);
        let sockets: Vec<PathBuf> = (0..BINDS)
            .map(|index| scratch.0.join(format!("{index}.sock")))
            .collect();
        let listeners: Vec<UnixListener> = std::thread::scope(|scope| {
            let handles: Vec<_> = sockets
                .iter()
                .map(|socket| {
                    let handle = runtime.handle().clone();
                    let barrier = &barrier;
                    scope.spawn(move || {
                        let _entered = handle.enter();
                        barrier.wait();
                        bind(socket).unwrap()
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect()
        });
        for socket in &sockets {
            assert_eq!(mode(socket), 0o600, "{}", socket.display());
        }
        assert_eq!(umask(), before);
        assert!(scratch.leftovers().is_empty(), "{:?}", scratch.leftovers());
        drop(listeners);
    }

    #[test]
    fn longest_socket_path_binds() {
        let scratch = Scratch::new("long");
        let prefix = scratch.0.as_os_str().len() + 1;
        assert!(prefix < SUN_PATH_MAX, "{}", scratch.0.display());
        let socket = scratch.0.join("s".repeat(SUN_PATH_MAX - prefix));
        assert_eq!(socket.as_os_str().len(), SUN_PATH_MAX);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();
        let _entered = runtime.enter();
        let listener = bind(&socket).unwrap();
        assert_eq!(mode(&socket), 0o600);
        assert!(scratch.leftovers().is_empty(), "{:?}", scratch.leftovers());
        drop(listener);
    }
}
