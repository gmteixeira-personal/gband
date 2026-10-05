use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::Result;
use gband_core::geometry::Size;
use gband_core::layout::LayoutOptions;
use gband_protocol::{SessionName, SessionSummary};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tracing::Instrument;

use crate::event::{Bus, Published};
use crate::session::{self, Command, Session, SessionConfig, State};

pub struct SessionHandle {
    id: u64,
    pub name: SessionName,
    pub commands: mpsc::UnboundedSender<Command>,
    pub state: watch::Receiver<Arc<State>>,
    pub changed: watch::Receiver<u64>,
    pub ended: watch::Receiver<bool>,
    pub clients: AtomicU32,
    pub events: Bus,
}

impl SessionHandle {
    pub fn command(&self, command: Command) {
        if self.commands.send(command).is_err() {
            tracing::debug!("command dropped because the session has ended");
        }
    }

    fn is_ending(&self) -> bool {
        *self.ended.borrow() || self.state.borrow().layout.is_empty()
    }
}

pub enum Request {
    Attach {
        name: SessionName,
        cwd: PathBuf,
        area: Size,
        reply: oneshot::Sender<Result<Arc<SessionHandle>>>,
    },
    List {
        reply: oneshot::Sender<Vec<SessionSummary>>,
    },
    Kill {
        name: SessionName,
        reply: oneshot::Sender<Option<Arc<SessionHandle>>>,
    },
    Ended {
        name: SessionName,
        id: u64,
    },
}

pub struct Registry {
    sessions: BTreeMap<SessionName, Arc<SessionHandle>>,
    program: Vec<OsString>,
    socket: PathBuf,
    requests: mpsc::UnboundedSender<Request>,
    events: broadcast::Sender<Published>,
    options: watch::Receiver<LayoutOptions>,
    next_id: u64,
}

impl Registry {
    pub fn new(
        program: Vec<OsString>,
        socket: PathBuf,
        requests: mpsc::UnboundedSender<Request>,
        events: broadcast::Sender<Published>,
        options: watch::Receiver<LayoutOptions>,
    ) -> Self {
        Self {
            sessions: BTreeMap::new(),
            program,
            socket,
            requests,
            events,
            options,
            next_id: 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn create(
        &mut self,
        name: SessionName,
        cwd: PathBuf,
        area: Size,
    ) -> Result<Arc<SessionHandle>> {
        let id = self.next_id;
        self.next_id += 1;
        let span = tracing::info_span!("session", name = %name);
        let (exits_tx, exits) = mpsc::unbounded_channel();
        let session = span.in_scope(|| {
            Session::start(
                SessionConfig {
                    name: name.clone(),
                    program: self.program.clone(),
                    cwd,
                    socket: self.socket.clone(),
                    area,
                    events: Bus::new(self.events.clone(), name.clone()),
                    options: self.options.clone(),
                },
                exits_tx,
            )
        })?;
        let (commands_tx, commands) = mpsc::unbounded_channel();
        let (ended_tx, ended) = watch::channel(false);
        let handle = Arc::new(SessionHandle {
            id,
            name: name.clone(),
            commands: commands_tx,
            state: session.state(),
            changed: session.changed(),
            ended,
            clients: AtomicU32::new(0),
            events: session.events(),
        });
        let requests = self.requests.clone();
        let ended_name = name.clone();
        tokio::spawn(
            async move {
                session::drive(session, commands, exits).await;
                let _ = requests.send(Request::Ended {
                    name: ended_name,
                    id,
                });
                ended_tx.send_replace(true);
            }
            .instrument(span),
        );
        tracing::info!(session = %name, "session created");
        self.sessions.insert(name, Arc::clone(&handle));
        Ok(handle)
    }

    pub fn handle(&mut self, request: Request) {
        match request {
            Request::Attach {
                name,
                cwd,
                area,
                reply,
            } => {
                let handle = match self.live(&name) {
                    Some(handle) => Ok(Arc::clone(handle)),
                    None => self.create(name, cwd, area),
                };
                let _ = reply.send(handle);
            }
            Request::List { reply } => {
                let sessions = self
                    .sessions
                    .values()
                    .filter(|handle| !handle.is_ending())
                    .map(|handle| SessionSummary {
                        name: handle.name.clone(),
                        panes: handle.state.borrow().layout.panes().count() as u32,
                        clients: handle.clients.load(Ordering::Acquire),
                    })
                    .collect();
                let _ = reply.send(sessions);
            }
            Request::Kill { name, reply } => {
                let handle = self.live(&name).map(Arc::clone);
                if let Some(handle) = &handle {
                    tracing::info!(session = %name, "killing the session");
                    handle.command(Command::CloseAll);
                }
                let _ = reply.send(handle);
            }
            Request::Ended { name, id } => {
                if self
                    .sessions
                    .get(&name)
                    .is_some_and(|handle| handle.id == id)
                {
                    self.sessions.remove(&name);
                    tracing::info!(session = %name, "session removed");
                }
            }
        }
    }

    pub fn terminate(&self) {
        for handle in self.sessions.values() {
            handle.command(Command::CloseAll);
        }
    }

    fn live(&self, name: &SessionName) -> Option<&Arc<SessionHandle>> {
        self.sessions.get(name).filter(|handle| !handle.is_ending())
    }
}
