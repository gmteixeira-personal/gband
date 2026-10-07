use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, bail};
use gband_core::geometry::{Size, boxes, tiles};
use gband_core::layout::{
    BandId, Layout, LayoutOptions, Program, Proportion, SessionAction, WindowContent, WindowId,
};
use gband_protocol::{SessionName, Value};
use portable_pty::{ChildKiller, ExitStatus};
use rustix::process::{Pid, Signal};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::Instant;
use tracing::Instrument;

use crate::event::{Bus, SessionEvent};
use crate::scripting::{Taps, signal_number};
use crate::window::{self, SpawnRequest, Window, WindowEntry, WindowExit};

const KILL_GRACE: Duration = Duration::from_secs(2);
const SETTLE: Duration = Duration::from_millis(100);
const NAME_CHECK: Duration = Duration::from_secs(1);
pub const INITIAL_AREA: Size = Size::new(80, 24);

pub struct State {
    pub layout: Layout,
    pub area: Size,
    pub windows: HashMap<WindowId, WindowEntry>,
}

pub enum Command {
    Area {
        size: Size,
        applied: Option<oneshot::Sender<()>>,
    },
    Action {
        client: u64,
        action: SessionAction,
        reply: Option<mpsc::UnboundedSender<Reply>>,
    },
    Shown {
        client: u64,
        windows: Vec<WindowId>,
    },
    Content {
        client: u64,
        window: WindowId,
        output: Vec<u8>,
    },
    Rename {
        window: WindowId,
        name: Option<String>,
    },
    Leave {
        client: u64,
    },
    CloseAll,
    Barrier(oneshot::Sender<()>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Focus(WindowId),
    Opened {
        request: u32,
        window: Option<WindowId>,
    },
    Result {
        call: u64,
        result: Result<Value, String>,
    },
}

struct Placement {
    band: BandId,
    after: Option<WindowId>,
    width: Option<Proportion>,
    floating: bool,
}

pub struct SessionConfig {
    pub name: SessionName,
    pub program: Vec<OsString>,
    pub cwd: PathBuf,
    pub socket: PathBuf,
    pub area: Size,
    pub events: Bus,
    pub options: watch::Receiver<LayoutOptions>,
    pub taps: Arc<Taps>,
}

struct Live {
    entry: WindowEntry,
    killer: Box<dyn ChildKiller + Send + Sync>,
    pid: Option<u32>,
    exit: watch::Receiver<Option<ExitStatus>>,
}

struct PluginWindow {
    owner: u64,
    entry: WindowEntry,
}

pub struct Session {
    config: SessionConfig,
    layout: Layout,
    area: Size,
    windows: HashMap<WindowId, Live>,
    plugins: HashMap<WindowId, PluginWindow>,
    state: watch::Sender<Arc<State>>,
    changed: Arc<watch::Sender<u64>>,
    exits: mpsc::UnboundedSender<WindowExit>,
    last_status: Option<ExitStatus>,
    shown: HashMap<u64, HashSet<WindowId>>,
    settle_at: Option<Instant>,
}

impl Session {
    pub fn start(config: SessionConfig, exits: mpsc::UnboundedSender<WindowExit>) -> Result<Self> {
        let area = config.area;
        let mut session = Self {
            config,
            layout: Layout::new(),
            area,
            windows: HashMap::new(),
            plugins: HashMap::new(),
            state: watch::Sender::new(Arc::new(State {
                layout: Layout::new(),
                area,
                windows: HashMap::new(),
            })),
            changed: Arc::new(watch::Sender::new(0)),
            exits,
            last_status: None,
            shown: HashMap::new(),
            settle_at: None,
        };
        let band = session.layout.bands()[0].id;
        let placement = Placement {
            band,
            after: None,
            width: None,
            floating: false,
        };
        session.open(placement, None)?;
        session.publish();
        Ok(session)
    }

    pub fn state(&self) -> watch::Receiver<Arc<State>> {
        self.state.subscribe()
    }

    pub fn changed(&self) -> watch::Receiver<u64> {
        self.changed.subscribe()
    }

    pub fn events(&self) -> Bus {
        self.config.events.clone()
    }

    pub fn is_over(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn handle(&mut self, command: Command) {
        match command {
            Command::Area { size, applied } => {
                if size.cols > 0 && size.rows > 0 && size != self.area {
                    self.area = size;
                    self.publish();
                }
                if let Some(applied) = applied {
                    let _ = applied.send(());
                }
            }
            Command::Action {
                client,
                action,
                reply,
            } => self.act(client, action, reply),
            Command::Shown { client, windows } => self.show(client, windows),
            Command::Content {
                client,
                window,
                output,
            } => match self.plugins.get(&window) {
                Some(plugin) if plugin.owner == client => plugin.entry.window.replace(&output),
                _ => {
                    tracing::debug!(window = %window, "ignoring content for a window the client does not own")
                }
            },
            Command::Rename { window, name } => self.rename(window, name),
            Command::Leave { client } => {
                let owned: Vec<WindowId> = self
                    .plugins
                    .iter()
                    .filter(|(_, plugin)| plugin.owner == client)
                    .map(|(&window, _)| window)
                    .collect();
                for window in owned {
                    self.close(window);
                }
            }
            Command::CloseAll => self.terminate(),
            Command::Barrier(reached) => {
                let _ = reached.send(());
            }
        }
    }

    fn rename(&self, window: WindowId, name: Option<String>) {
        let Some(live) = self.windows.get(&window) else {
            tracing::debug!(window = %window, "ignoring a rename of a window that runs no program");
            return;
        };
        let name = name
            .map(|name| name.trim().to_owned())
            .filter(|name| !name.is_empty());
        live.entry.window.rename(name);
    }

    pub fn refresh_names(&self) {
        for live in self.windows.values() {
            live.entry.window.refresh_name();
        }
    }

    fn show(&mut self, client: u64, windows: Vec<WindowId>) {
        let windows: HashSet<WindowId> = windows
            .into_iter()
            .filter(|&window| self.layout.contains(window))
            .collect();
        let previous = if windows.is_empty() {
            self.shown.remove(&client)
        } else {
            self.shown.insert(client, windows.clone())
        };
        if previous.unwrap_or_default() != windows {
            self.unsettle();
        }
    }

    fn unsettle(&mut self) {
        self.settle_at = Some(Instant::now() + SETTLE);
    }

    fn settle(&mut self) {
        self.settle_at = None;
        let shown: HashSet<WindowId> = self.shown.values().flatten().copied().collect();
        for band in self.layout.bands() {
            let tiled = tiles(band, self.area)
                .into_iter()
                .map(|tile| (tile.window, tile.terminal_size()));
            let floating = boxes(band, self.area)
                .into_iter()
                .map(|placed| (placed.window, placed.terminal_size()));
            for (window, size) in tiled.chain(floating) {
                if !shown.contains(&window) {
                    continue;
                }
                if let Some(entry) = self.entry(window) {
                    entry.window.resize(size);
                }
            }
        }
    }

    pub fn exited(&mut self, exit: WindowExit) {
        tracing::info!(window = %exit.window, "program exited: {}", exit.status);
        let signal = exit.status.signal().map(|name| (name, signal_number(name)));
        if let Some((name, None)) = signal {
            tracing::debug!("no signal number is known for `{name}`");
        }
        self.config.events.send(SessionEvent::WindowExited {
            window: exit.window,
            code: signal.is_none().then(|| exit.status.exit_code()),
            signal: signal.and_then(|(_, number)| number),
        });
        self.last_status = Some(exit.status);
        self.windows.remove(&exit.window);
        let mut events = self.layout.remove(exit.window);
        if self.windows.is_empty() {
            for (window, _) in self.plugins.drain() {
                events.extend(self.layout.remove(window));
            }
        }
        if !events.is_empty() {
            self.config.events.layout(events);
            self.publish();
        }
    }

    pub fn terminate(&mut self) {
        let windows: Vec<_> = self.windows.keys().copied().collect();
        for window in windows {
            self.close(window);
        }
    }

    fn act(
        &mut self,
        client: u64,
        action: SessionAction,
        reply: Option<mpsc::UnboundedSender<Reply>>,
    ) {
        let respond = |message: Reply| {
            if let Some(reply) = &reply {
                let _ = reply.send(message);
            }
        };
        match action {
            SessionAction::OpenWindow {
                band,
                after,
                width,
                floating,
                focus,
                content,
            } => {
                let placement = Placement {
                    band,
                    after,
                    width,
                    floating,
                };
                let opened = match content {
                    WindowContent::Program(ref program) => self.open(placement, program.as_ref()),
                    WindowContent::Plugin { .. } => Ok(self.open_plugin(client, placement)),
                };
                if let Ok(Some(_)) = opened {
                    self.publish();
                }
                if let (WindowContent::Plugin { request }, Ok(window)) = (&content, &opened) {
                    respond(Reply::Opened {
                        request: *request,
                        window: *window,
                    });
                }
                match opened {
                    Ok(Some(window)) => {
                        if focus {
                            respond(Reply::Focus(window));
                        }
                    }
                    Ok(None) => tracing::debug!("ignoring an open window action on a stale target"),
                    Err(error) => tracing::warn!("cannot open a window: {error:#}"),
                }
            }
            SessionAction::CloseWindow(window) => self.close(window),
            other => {
                let options = self.config.options.borrow().clone();
                let events = self.layout.apply(other, self.area, &options);
                if !events.is_empty() {
                    self.config.events.layout(events);
                    self.publish();
                }
            }
        }
    }

    fn placeable(&self, placement: &Placement) -> bool {
        let after = placement.after.filter(|_| !placement.floating);
        self.layout.can_open(placement.band, after)
    }

    fn place(&mut self, placement: &Placement) -> Option<WindowId> {
        if !self.placeable(placement) {
            return None;
        }
        let id = self.layout.allocate_window();
        let options = self.config.options.borrow().clone();
        let events = if placement.floating {
            self.layout
                .open_floating(id, placement.band, placement.width, self.area, &options)
        } else {
            self.layout.open(
                id,
                placement.band,
                placement.after,
                placement.width,
                &options,
            )
        };
        self.config.events.layout(events);
        Some(id)
    }

    fn open_plugin(&mut self, client: u64, placement: Placement) -> Option<WindowId> {
        let id = self.place(&placement)?;
        let size = self.terminal_size(id).expect("an opened window has a tile");
        let entry = Window::blank(size, &self.changed);
        self.plugins.insert(
            id,
            PluginWindow {
                owner: client,
                entry,
            },
        );
        tracing::info!(window = %id, client, "opened a plugin window");
        Some(id)
    }

    fn open(
        &mut self,
        placement: Placement,
        program: Option<&Program>,
    ) -> Result<Option<WindowId>> {
        if !self.placeable(&placement) {
            return Ok(None);
        }
        let argv = match program {
            None => self.config.program.clone(),
            Some(Program::CommandLine(line)) => {
                vec![crate::user_shell(), "-c".into(), line.into()]
            }
            Some(Program::Argv(argv)) if argv.is_empty() => bail!("the argument list is empty"),
            Some(Program::Argv(argv)) => argv.iter().map(OsString::from).collect(),
        };
        let id = self.place(&placement).expect("checked by can_open");
        let size = self.terminal_size(id).expect("an opened window has a tile");
        let request = SpawnRequest {
            id,
            program: &argv,
            cwd: &self.config.cwd,
            socket: &self.config.socket,
            session: &self.config.name,
            size,
            taps: &self.config.taps,
        };
        match window::spawn(request, &self.changed, self.exits.clone()) {
            Ok(spawned) => {
                self.windows.insert(
                    id,
                    Live {
                        entry: spawned.entry,
                        killer: spawned.killer,
                        pid: spawned.pid,
                        exit: spawned.exit,
                    },
                );
                Ok(Some(id))
            }
            Err(error) => {
                let events = self.layout.remove(id);
                self.config.events.layout(events);
                Err(error)
            }
        }
    }

    fn close(&mut self, window: WindowId) {
        if self.plugins.remove(&window).is_some() {
            tracing::info!(window = %window, "closing a plugin window");
            let events = self.layout.remove(window);
            self.config.events.layout(events);
            self.publish();
            return;
        }
        let Some(live) = self.windows.get_mut(&window) else {
            return;
        };
        tracing::info!(window = %window, "hanging up the program");
        if let Err(error) = live.killer.kill() {
            tracing::warn!("cannot send SIGHUP to the program: {error:#}");
        }
        tokio::spawn(
            kill_if_running(live.exit.clone(), Arc::clone(&live.entry.window), live.pid)
                .in_current_span(),
        );
    }

    fn entry(&self, window: WindowId) -> Option<&WindowEntry> {
        self.windows
            .get(&window)
            .map(|live| &live.entry)
            .or_else(|| self.plugins.get(&window).map(|plugin| &plugin.entry))
    }

    fn terminal_size(&self, window: WindowId) -> Option<Size> {
        self.layout.bands().iter().find_map(|band| {
            let tiled = tiles(band, self.area)
                .into_iter()
                .find(|tile| tile.window == window)
                .map(|tile| tile.terminal_size());
            tiled.or_else(|| {
                boxes(band, self.area)
                    .into_iter()
                    .find(|placed| placed.window == window)
                    .map(|placed| placed.terminal_size())
            })
        })
    }

    fn publish(&mut self) {
        self.unsettle();
        let windows = self
            .windows
            .iter()
            .map(|(&id, live)| (id, live.entry.clone()))
            .chain(
                self.plugins
                    .iter()
                    .map(|(&id, plugin)| (id, plugin.entry.clone())),
            )
            .collect();
        self.state.send_replace(Arc::new(State {
            layout: self.layout.clone(),
            area: self.area,
            windows,
        }));
        self.changed.send_modify(|generation| *generation += 1);
    }
}

pub async fn drive(
    mut session: Session,
    mut commands: mpsc::UnboundedReceiver<Command>,
    mut exits: mpsc::UnboundedReceiver<WindowExit>,
) {
    let mut names = tokio::time::interval(NAME_CHECK);
    names.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    while !session.is_over() {
        let settle_at = session.settle_at;
        tokio::select! {
            Some(command) = commands.recv() => session.handle(command),
            Some(exit) = exits.recv() => session.exited(exit),
            _ = names.tick() => session.refresh_names(),
            () = async { tokio::time::sleep_until(settle_at.unwrap()).await }, if settle_at.is_some() => {
                session.settle();
            }
        }
    }
    match session.last_status {
        Some(status) => tracing::info!("session ended, last program exited: {status}"),
        None => tracing::info!("session ended"),
    }
}

async fn kill_if_running(
    mut exit: watch::Receiver<Option<ExitStatus>>,
    window: Arc<Window>,
    pid: Option<u32>,
) {
    let exited = tokio::time::timeout(KILL_GRACE, exit.wait_for(Option::is_some)).await;
    if exited.is_ok() {
        return;
    }
    tracing::warn!("program still running after SIGHUP, sending SIGKILL");
    if let Some(group) = window.foreground_group().and_then(Pid::from_raw) {
        let _ = rustix::process::kill_process_group(group, Signal::KILL);
    }
    if let Some(program) = pid.and_then(|pid| Pid::from_raw(pid as i32)) {
        let _ = rustix::process::kill_process(program, Signal::KILL);
    }
}
