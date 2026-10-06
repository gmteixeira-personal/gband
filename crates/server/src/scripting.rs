use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;

use gband_core::event::LayoutEvent;
use gband_core::layout::WindowId;
use gband_lua::server::{Caller, Event};
use gband_lua::{Config, ConfigError, Dispatch, Outcome};
use gband_protocol::{Requirement, SessionName, Value};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{Semaphore, broadcast, oneshot};

use crate::event::{Published, SessionEvent};
use crate::hub::{Hub, HubHost};
use crate::session::{Command, Reply};

pub const OUTPUT_BUDGET: usize = 4 * 1024 * 1024;
pub const NOTICE_BUDGET: usize = 1024;
const LUA_CLIENT: u64 = 0;

pub(crate) enum Input {
    Bus(Published),
    Output {
        session: SessionName,
        window: WindowId,
        bytes: Vec<u8>,
    },
    Notice {
        session: SessionName,
        window: WindowId,
        client: u64,
    },
    Call {
        session: SessionName,
        client: u64,
        call: u64,
        name: String,
        args: Value,
        replies: UnboundedSender<Reply>,
    },
    Reload(Result<Config, ConfigError>),
    Eval {
        source: String,
        args: Vec<Value>,
        reply: oneshot::Sender<Result<Vec<Value>, String>>,
    },
    Barrier(oneshot::Sender<()>),
}

pub struct Scripting {
    config: Config,
    error: Option<ConfigError>,
    sender: mpsc::Sender<Input>,
    receiver: mpsc::Receiver<Input>,
}

#[derive(Clone)]
pub struct Reloader(mpsc::Sender<Input>);

impl Reloader {
    pub fn reload(&self, result: Result<Config, ConfigError>) {
        let _ = self.0.send(Input::Reload(result));
    }
}

impl Scripting {
    pub fn new(config: Config, error: Option<ConfigError>) -> (Self, Reloader) {
        let (sender, receiver) = mpsc::channel();
        let reloader = Reloader(sender.clone());
        (
            Self {
                config,
                error,
                sender,
                receiver,
            },
            reloader,
        )
    }
}

pub struct Taps {
    sender: Option<mpsc::Sender<Input>>,
    output: AtomicBool,
    input: AtomicBool,
    bytes: AtomicUsize,
    dropped: AtomicU64,
    notices: AtomicUsize,
    dropped_notices: AtomicU64,
}

impl Taps {
    pub fn idle() -> Arc<Self> {
        Arc::new(Self::with(None))
    }

    fn with(sender: Option<mpsc::Sender<Input>>) -> Self {
        Self {
            sender,
            output: AtomicBool::new(false),
            input: AtomicBool::new(false),
            bytes: AtomicUsize::new(0),
            dropped: AtomicU64::new(0),
            notices: AtomicUsize::new(0),
            dropped_notices: AtomicU64::new(0),
        }
    }

    pub fn output(&self, session: &SessionName, window: WindowId, bytes: &[u8]) {
        let Some(sender) = &self.sender else {
            return;
        };
        if !self.output.load(Ordering::Acquire) {
            return;
        }
        let pending = self.bytes.fetch_add(bytes.len(), Ordering::AcqRel);
        if pending + bytes.len() > OUTPUT_BUDGET {
            self.bytes.fetch_sub(bytes.len(), Ordering::AcqRel);
            self.dropped
                .fetch_add(bytes.len() as u64, Ordering::Relaxed);
            return;
        }
        let input = Input::Output {
            session: session.clone(),
            window,
            bytes: bytes.to_vec(),
        };
        if sender.send(input).is_err() {
            self.bytes.fetch_sub(bytes.len(), Ordering::AcqRel);
        }
    }

    pub fn notice(&self, session: &SessionName, window: WindowId, client: u64) {
        let Some(sender) = &self.sender else {
            return;
        };
        if !self.input.load(Ordering::Acquire) {
            return;
        }
        if self.notices.fetch_add(1, Ordering::AcqRel) >= NOTICE_BUDGET {
            self.notices.fetch_sub(1, Ordering::AcqRel);
            self.dropped_notices.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let input = Input::Notice {
            session: session.clone(),
            window,
            client,
        };
        if sender.send(input).is_err() {
            self.notices.fetch_sub(1, Ordering::AcqRel);
        }
    }

    pub(crate) fn call(&self, input: Input) -> bool {
        self.sender
            .as_ref()
            .is_some_and(|sender| sender.send(input).is_ok())
    }

    fn want(&self, config: &Config) {
        self.output
            .store(config.runtime.handles("WindowOutput"), Ordering::Release);
        self.input
            .store(config.runtime.handles("WindowInput"), Ordering::Release);
    }
}

pub(crate) fn start(
    scripting: Scripting,
    hub: Arc<Hub>,
    bus: &broadcast::Sender<Published>,
) -> Arc<Taps> {
    let Scripting {
        config,
        error,
        sender,
        receiver,
    } = scripting;
    let taps = Arc::new(Taps::with(Some(sender.clone())));
    let gate = Arc::new(Semaphore::new(1));
    let (barriers_tx, barriers) = tokio::sync::mpsc::unbounded_channel();
    hub.settling.set_forward(barriers_tx);
    tokio::spawn(forward(
        bus.subscribe(),
        sender,
        Arc::clone(&gate),
        barriers,
    ));
    let worker = Worker {
        config,
        hub,
        taps: Arc::clone(&taps),
        gate,
    };
    let spawned = thread::Builder::new()
        .name("gband-lua".to_owned())
        .spawn(move || worker.run(error, receiver));
    if let Err(error) = spawned {
        tracing::warn!("cannot start the Lua thread: {error}");
    }
    taps
}

async fn forward(
    mut bus: broadcast::Receiver<Published>,
    sender: mpsc::Sender<Input>,
    gate: Arc<Semaphore>,
    mut barriers: UnboundedReceiver<oneshot::Sender<()>>,
) {
    loop {
        let Ok(permit) = gate.acquire().await else {
            return;
        };
        permit.forget();
        tokio::select! {
            received = bus.recv() => match received {
                Ok(published) => {
                    if sender.send(Input::Bus(published)).is_err() {
                        return;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(missed)) => {
                    tracing::warn!("the Lua handlers fell behind and dropped {missed} session events");
                    gate.add_permits(1);
                }
                Err(broadcast::error::RecvError::Closed) => return,
            },
            Some(reached) = barriers.recv() => {
                if !drain(&mut bus, &sender, &gate).await
                    || sender.send(Input::Barrier(reached)).is_err()
                {
                    return;
                }
            }
        }
    }
}

async fn drain(
    bus: &mut broadcast::Receiver<Published>,
    sender: &mpsc::Sender<Input>,
    gate: &Semaphore,
) -> bool {
    let mut held = true;
    loop {
        match bus.try_recv() {
            Ok(published) => {
                if !held {
                    let Ok(permit) = gate.acquire().await else {
                        return false;
                    };
                    permit.forget();
                }
                held = false;
                if sender.send(Input::Bus(published)).is_err() {
                    return false;
                }
            }
            Err(broadcast::error::TryRecvError::Lagged(missed)) => {
                tracing::warn!("the Lua handlers fell behind and dropped {missed} session events");
            }
            Err(broadcast::error::TryRecvError::Empty) => break,
            Err(broadcast::error::TryRecvError::Closed) => return false,
        }
    }
    if held {
        gate.add_permits(1);
    }
    true
}

struct Worker {
    config: Config,
    hub: Arc<Hub>,
    taps: Arc<Taps>,
    gate: Arc<Semaphore>,
}

fn requirements(config: &Config) -> Vec<Requirement> {
    config
        .plugins
        .iter()
        .filter(|plugin| !plugin.failed)
        .filter_map(|plugin| {
            Some(Requirement {
                plugin: plugin.name.clone(),
                requirement: plugin.client.clone()?,
            })
        })
        .collect()
}

pub fn signal_number(name: &str) -> Option<i32> {
    if let Some((_, number)) = name.rsplit_once(": ")
        && let Ok(number) = number.trim().parse()
    {
        return Some(number);
    }
    const NAMES: [&str; 31] = [
        "Hangup",
        "Interrupt",
        "Quit",
        "Illegal instruction",
        "Trace/breakpoint trap",
        "Aborted",
        "Bus error",
        "Floating point exception",
        "Killed",
        "User defined signal 1",
        "Segmentation fault",
        "User defined signal 2",
        "Broken pipe",
        "Alarm clock",
        "Terminated",
        "Stack fault",
        "Child exited",
        "Continued",
        "Stopped (signal)",
        "Stopped",
        "Stopped (tty input)",
        "Stopped (tty output)",
        "Urgent I/O condition",
        "CPU time limit exceeded",
        "File size limit exceeded",
        "Virtual timer expired",
        "Profiling timer expired",
        "Window changed",
        "I/O possible",
        "Power failure",
        "Bad system call",
    ];
    NAMES
        .iter()
        .position(|known| *known == name)
        .map(|index| index as i32 + 1)
}

impl Worker {
    fn run(mut self, error: Option<ConfigError>, inputs: mpsc::Receiver<Input>) {
        let errors: Vec<ConfigError> = error
            .into_iter()
            .chain(self.config.errors.iter().cloned())
            .collect();
        self.loaded(&errors);
        while let Ok(input) = inputs.recv() {
            self.handle(input);
        }
    }

    fn loaded(&mut self, errors: &[ConfigError]) {
        self.config
            .runtime
            .set_host(Arc::new(HubHost(Arc::clone(&self.hub))));
        self.taps.want(&self.config);
        self.hub.set_requirements(requirements(&self.config));
        self.hub.clear_error();
        self.report(errors);
    }

    fn report(&self, errors: &[ConfigError]) {
        for error in errors {
            tracing::warn!("server configuration error: {error}");
            self.hub.report(error.to_string());
        }
    }

    fn handle(&mut self, input: Input) {
        match input {
            Input::Bus(published) => {
                self.gate.add_permits(1);
                self.bus(published);
            }
            Input::Output {
                session,
                window,
                bytes,
            } => {
                self.taps.bytes.fetch_sub(bytes.len(), Ordering::AcqRel);
                let dropped = self.taps.dropped.swap(0, Ordering::Relaxed);
                if dropped > 0 {
                    tracing::warn!(
                        "dropped {dropped} bytes of window output the Lua handlers could not keep up with"
                    );
                }
                self.emit(&Event::WindowOutput {
                    session: session.as_str().to_owned(),
                    window,
                    data: bytes,
                });
            }
            Input::Notice {
                session,
                window,
                client,
            } => {
                self.taps.notices.fetch_sub(1, Ordering::AcqRel);
                let dropped = self.taps.dropped_notices.swap(0, Ordering::Relaxed);
                if dropped > 0 {
                    tracing::warn!(
                        "dropped {dropped} input notices the Lua handlers could not keep up with"
                    );
                }
                self.emit(&Event::WindowInput {
                    session: session.as_str().to_owned(),
                    window,
                    client,
                });
            }
            Input::Call {
                session,
                client,
                call,
                name,
                args,
                replies,
            } => {
                let focus = replies.clone();
                let caller = Caller {
                    session: session.as_str().to_owned(),
                    client,
                    focus: Arc::new(move |window| {
                        let _ = focus.send(Reply::Focus(window));
                    }),
                };
                let (result, outcome) = self.config.runtime.command(&name, args, Some(caller));
                if let Err(reason) = &result {
                    tracing::warn!(client, "the command `{name}` failed: {reason}");
                }
                self.apply(outcome);
                let _ = replies.send(Reply::Result { call, result });
            }
            Input::Reload(Ok(config)) => {
                tracing::info!("server configuration reloaded");
                self.config = config;
                let errors = self.config.errors.clone();
                self.loaded(&errors);
                self.emit(&Event::ConfigReloaded);
            }
            Input::Reload(Err(error)) => self.report(&[error]),
            Input::Eval {
                source,
                args,
                reply,
            } => {
                let (answer, outcome) = self.config.runtime.eval(&source, &args);
                self.apply(outcome);
                let _ = reply.send(answer);
            }
            Input::Barrier(reached) => {
                let _ = reached.send(());
            }
        }
    }

    fn bus(&mut self, published: Published) {
        let Published { session, event } = published;
        let name = || session.as_str().to_owned();
        let event = match event {
            SessionEvent::Created => Event::SessionCreated { session: name() },
            SessionEvent::Ended => {
                self.hub.session_ended(&session);
                Event::SessionEnded { session: name() }
            }
            SessionEvent::Layout(LayoutEvent::WindowOpened { window, band }) => {
                self.hub.window_opened(&session, window);
                Event::WindowOpened {
                    session: name(),
                    window,
                    band,
                }
            }
            SessionEvent::Layout(LayoutEvent::WindowClosed { window, band }) => {
                self.hub.window_closed(&session, window);
                Event::WindowClosed {
                    session: name(),
                    window,
                    band,
                }
            }
            SessionEvent::Layout(_) => return,
            SessionEvent::WindowExited {
                window,
                code,
                signal,
            } => Event::WindowExited {
                session: name(),
                window,
                code,
                signal,
            },
            SessionEvent::ClientAttached { client } => Event::ClientAttached {
                session: name(),
                client,
            },
            SessionEvent::ClientDetached { client } => Event::ClientDetached {
                session: name(),
                client,
            },
        };
        self.emit(&event);
    }

    fn emit(&mut self, event: &Event) {
        let outcome = self.config.runtime.emit_server(event);
        self.apply(outcome);
    }

    fn apply(&mut self, outcome: Outcome) {
        for entry in outcome.dispatched {
            match entry {
                Dispatch::Targeted { session, action } => match self.hub.session(&session) {
                    Some(handle) => {
                        self.hub.settling.count();
                        handle.command(Command::Action {
                            client: LUA_CLIENT,
                            action,
                            reply: None,
                        })
                    }
                    None => {
                        tracing::debug!("ignoring an action for the absent session `{session}`")
                    }
                },
                other => tracing::debug!("ignoring a client effect in the server: {other:?}"),
            }
        }
        self.report(&outcome.errors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_numbers() {
        assert_eq!(signal_number("Killed"), Some(9));
        assert_eq!(signal_number("Hangup"), Some(1));
        assert_eq!(signal_number("Terminated"), Some(15));
        assert_eq!(signal_number("Killed: 9"), Some(9));
        assert_eq!(signal_number("Unknown signal 77"), None);
    }
}
