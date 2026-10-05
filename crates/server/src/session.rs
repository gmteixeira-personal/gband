use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, bail};
use gband_core::geometry::{Size, tiles};
use gband_core::layout::{BandId, Layout, LayoutOptions, PaneId, Program, SessionAction};
use gband_protocol::SessionName;
use portable_pty::{ChildKiller, ExitStatus};
use rustix::process::{Pid, Signal};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::Instant;
use tracing::Instrument;

use crate::event::{Bus, SessionEvent};
use crate::pane::{self, Pane, PaneEntry, PaneExit, SpawnRequest};

const KILL_GRACE: Duration = Duration::from_secs(2);
const SETTLE: Duration = Duration::from_millis(100);
pub const INITIAL_AREA: Size = Size::new(80, 24);

pub struct State {
    pub layout: Layout,
    pub area: Size,
    pub panes: HashMap<PaneId, PaneEntry>,
}

pub enum Command {
    Area {
        size: Size,
        applied: Option<oneshot::Sender<()>>,
    },
    Action {
        action: SessionAction,
        focus: Option<mpsc::UnboundedSender<PaneId>>,
    },
    Shown {
        client: u64,
        panes: Vec<PaneId>,
    },
    CloseAll,
}

pub struct SessionConfig {
    pub name: SessionName,
    pub program: Vec<OsString>,
    pub cwd: PathBuf,
    pub socket: PathBuf,
    pub area: Size,
    pub events: Bus,
    pub options: watch::Receiver<LayoutOptions>,
}

struct Live {
    entry: PaneEntry,
    killer: Box<dyn ChildKiller + Send + Sync>,
    pid: Option<u32>,
    exit: watch::Receiver<Option<ExitStatus>>,
}

pub struct Session {
    config: SessionConfig,
    layout: Layout,
    area: Size,
    panes: HashMap<PaneId, Live>,
    state: watch::Sender<Arc<State>>,
    changed: Arc<watch::Sender<u64>>,
    exits: mpsc::UnboundedSender<PaneExit>,
    last_status: Option<ExitStatus>,
    shown: HashMap<u64, HashSet<PaneId>>,
    settle_at: Option<Instant>,
}

impl Session {
    pub fn start(config: SessionConfig, exits: mpsc::UnboundedSender<PaneExit>) -> Result<Self> {
        let area = config.area;
        let mut session = Self {
            config,
            layout: Layout::new(),
            area,
            panes: HashMap::new(),
            state: watch::Sender::new(Arc::new(State {
                layout: Layout::new(),
                area,
                panes: HashMap::new(),
            })),
            changed: Arc::new(watch::Sender::new(0)),
            exits,
            last_status: None,
            shown: HashMap::new(),
            settle_at: None,
        };
        let band = session.layout.bands()[0].id;
        session.open(band, None, None)?;
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
        self.layout.is_empty()
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
            Command::Action { action, focus } => self.act(action, focus),
            Command::Shown { client, panes } => self.show(client, panes),
            Command::CloseAll => self.terminate(),
        }
    }

    fn show(&mut self, client: u64, panes: Vec<PaneId>) {
        let panes: HashSet<PaneId> = panes
            .into_iter()
            .filter(|&pane| self.layout.contains(pane))
            .collect();
        let previous = if panes.is_empty() {
            self.shown.remove(&client)
        } else {
            self.shown.insert(client, panes.clone())
        };
        if previous.unwrap_or_default() != panes {
            self.unsettle();
        }
    }

    fn unsettle(&mut self) {
        self.settle_at = Some(Instant::now() + SETTLE);
    }

    fn settle(&mut self) {
        self.settle_at = None;
        let shown: HashSet<PaneId> = self.shown.values().flatten().copied().collect();
        for band in self.layout.bands() {
            for tile in tiles(band, self.area) {
                if !shown.contains(&tile.pane) {
                    continue;
                }
                if let Some(live) = self.panes.get(&tile.pane) {
                    live.entry.pane.resize(tile.terminal_size());
                }
            }
        }
    }

    pub fn exited(&mut self, exit: PaneExit) {
        tracing::info!(pane = %exit.pane, "program exited: {}", exit.status);
        self.config.events.send(SessionEvent::PaneExited {
            pane: exit.pane,
            status: exit.status.to_string(),
        });
        self.last_status = Some(exit.status);
        self.panes.remove(&exit.pane);
        let events = self.layout.remove(exit.pane);
        if !events.is_empty() {
            self.config.events.layout(events);
            self.publish();
        }
    }

    pub fn terminate(&mut self) {
        let panes: Vec<_> = self.panes.keys().copied().collect();
        for pane in panes {
            self.close(pane);
        }
    }

    fn act(&mut self, action: SessionAction, focus: Option<mpsc::UnboundedSender<PaneId>>) {
        match action {
            SessionAction::OpenPane {
                band,
                after,
                program,
            } => match self.open(band, after, program.as_ref()) {
                Ok(Some(pane)) => {
                    self.publish();
                    if let Some(focus) = focus {
                        let _ = focus.send(pane);
                    }
                }
                Ok(None) => tracing::debug!("ignoring an open pane action on a stale target"),
                Err(error) => tracing::warn!("cannot open a pane: {error:#}"),
            },
            SessionAction::ClosePane(pane) => self.close(pane),
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

    fn open(
        &mut self,
        band: BandId,
        after: Option<PaneId>,
        program: Option<&Program>,
    ) -> Result<Option<PaneId>> {
        if !self.layout.can_open(band, after) {
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
        let id = self.layout.allocate_pane();
        let options = self.config.options.borrow().clone();
        let events = self.layout.open(id, band, after, &options);
        self.config.events.layout(events);
        let size = self.terminal_size(id).expect("an opened pane has a tile");
        let request = SpawnRequest {
            id,
            program: &argv,
            cwd: &self.config.cwd,
            socket: &self.config.socket,
            session: &self.config.name,
            size,
        };
        match pane::spawn(request, &self.changed, self.exits.clone()) {
            Ok(spawned) => {
                self.panes.insert(
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

    fn close(&mut self, pane: PaneId) {
        let Some(live) = self.panes.get_mut(&pane) else {
            return;
        };
        tracing::info!(pane = %pane, "hanging up the program");
        if let Err(error) = live.killer.kill() {
            tracing::warn!("cannot send SIGHUP to the program: {error:#}");
        }
        tokio::spawn(
            kill_if_running(live.exit.clone(), Arc::clone(&live.entry.pane), live.pid)
                .in_current_span(),
        );
    }

    fn terminal_size(&self, pane: PaneId) -> Option<Size> {
        self.layout.bands().iter().find_map(|band| {
            tiles(band, self.area)
                .into_iter()
                .find(|tile| tile.pane == pane)
                .map(|tile| tile.terminal_size())
        })
    }

    fn publish(&mut self) {
        self.unsettle();
        let panes = self
            .panes
            .iter()
            .map(|(&id, live)| (id, live.entry.clone()))
            .collect();
        self.state.send_replace(Arc::new(State {
            layout: self.layout.clone(),
            area: self.area,
            panes,
        }));
        self.changed.send_modify(|generation| *generation += 1);
    }
}

pub async fn drive(
    mut session: Session,
    mut commands: mpsc::UnboundedReceiver<Command>,
    mut exits: mpsc::UnboundedReceiver<PaneExit>,
) {
    while !session.is_over() {
        let settle_at = session.settle_at;
        tokio::select! {
            Some(command) = commands.recv() => session.handle(command),
            Some(exit) = exits.recv() => session.exited(exit),
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
