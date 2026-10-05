use gband_core::event::LayoutEvent;
use gband_core::layout::PaneId;
use gband_protocol::SessionName;
use tokio::sync::broadcast::{self, error::RecvError};

pub const CAPACITY: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionEvent {
    Created,
    Ended,
    Layout(LayoutEvent),
    PaneExited {
        pane: PaneId,
        code: Option<u32>,
        signal: Option<i32>,
    },
    ClientAttached {
        client: u64,
    },
    ClientDetached {
        client: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Published {
    pub session: SessionName,
    pub event: SessionEvent,
}

#[derive(Clone)]
pub struct Bus {
    sender: broadcast::Sender<Published>,
    session: SessionName,
}

impl Bus {
    pub fn new(sender: broadcast::Sender<Published>, session: SessionName) -> Self {
        Self { sender, session }
    }

    pub fn send(&self, event: SessionEvent) {
        let _ = self.sender.send(Published {
            session: self.session.clone(),
            event,
        });
    }

    pub fn layout(&self, events: impl IntoIterator<Item = LayoutEvent>) {
        for event in events {
            self.send(SessionEvent::Layout(event));
        }
    }
}

pub fn channel() -> broadcast::Sender<Published> {
    broadcast::Sender::new(CAPACITY)
}

pub async fn log(mut events: broadcast::Receiver<Published>) {
    loop {
        match events.recv().await {
            Ok(Published { session, event }) => {
                tracing::debug!(session = %session, "session event: {event:?}");
            }
            Err(RecvError::Lagged(missed)) => {
                tracing::warn!("the event log missed {missed} session events");
            }
            Err(RecvError::Closed) => return,
        }
    }
}
