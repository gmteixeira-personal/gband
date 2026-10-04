use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use gband_core::geometry::{Size, tiles};
use gband_core::layout::{Layout, PaneId, SessionAction, WorkspaceId};
use gband_protocol::SessionName;
use portable_pty::{ChildKiller, ExitStatus};
use rustix::process::{Pid, Signal};
use tokio::sync::{mpsc, oneshot, watch};
use tracing::Instrument;

use crate::pane::{self, Pane, PaneEntry, PaneExit, SpawnRequest};

const KILL_GRACE: Duration = Duration::from_secs(2);
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
    CloseAll,
}

pub struct SessionConfig {
    pub name: SessionName,
    pub program: Vec<OsString>,
    pub cwd: PathBuf,
    pub socket: PathBuf,
    pub area: Size,
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
        };
        let workspace = session.layout.workspaces()[0].id;
        session.open(workspace, None)?;
        session.publish();
        Ok(session)
    }

    pub fn state(&self) -> watch::Receiver<Arc<State>> {
        self.state.subscribe()
    }

    pub fn changed(&self) -> watch::Receiver<u64> {
        self.changed.subscribe()
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
            Command::CloseAll => self.terminate(),
        }
    }

    pub fn exited(&mut self, exit: PaneExit) {
        tracing::info!(pane = %exit.pane, "program exited: {}", exit.status);
        self.last_status = Some(exit.status);
        self.panes.remove(&exit.pane);
        if self.layout.remove(exit.pane) {
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
            SessionAction::OpenPane { workspace, after } => match self.open(workspace, after) {
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
                if self.layout.apply(other) {
                    self.publish();
                }
            }
        }
    }

    fn open(&mut self, workspace: WorkspaceId, after: Option<PaneId>) -> Result<Option<PaneId>> {
        if !self.layout.can_open(workspace, after) {
            return Ok(None);
        }
        let id = self.layout.allocate_pane();
        self.layout.open(id, workspace, after);
        let size = self.terminal_size(id).expect("an opened pane has a tile");
        let request = SpawnRequest {
            id,
            program: &self.config.program,
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
                self.layout.remove(id);
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
        self.layout.workspaces().iter().find_map(|workspace| {
            tiles(workspace, self.area)
                .into_iter()
                .find(|tile| tile.pane == pane)
                .map(|tile| tile.terminal_size())
        })
    }

    fn publish(&mut self) {
        for workspace in self.layout.workspaces() {
            for tile in tiles(workspace, self.area) {
                if let Some(live) = self.panes.get(&tile.pane) {
                    live.entry.pane.resize(tile.terminal_size());
                }
            }
        }
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
        tokio::select! {
            Some(command) = commands.recv() => session.handle(command),
            Some(exit) = exits.recv() => session.exited(exit),
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
