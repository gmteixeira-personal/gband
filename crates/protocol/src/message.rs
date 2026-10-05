use gband_core::input::Key;
use gband_core::layout::{Layout, PaneId, SessionAction};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::session::{SessionName, SessionSummary};

pub const PROTOCOL_VERSION: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hello {
    pub version: u32,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelloReply {
    Accepted { version: u32 },
    Rejected { version: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientMessage {
    Attach { session: SessionName, cwd: PathBuf },
    ListSessions,
    KillSession { session: SessionName },
    Key { pane: PaneId, key: Key },
    Paste { pane: PaneId, text: String },
    Resize { cols: u16, rows: u16 },
    Action(SessionAction),
    Detach,
    Shown(Vec<PaneId>),
    Content { pane: PaneId, output: Vec<u8> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMessage {
    Info {
        pid: u32,
        executable: ExecutableId,
    },
    Layout {
        cols: u16,
        rows: u16,
        layout: Layout,
    },
    Snapshot {
        pane: PaneId,
        cols: u16,
        rows: u16,
        contents: Vec<u8>,
    },
    Update {
        pane: PaneId,
        contents: Vec<u8>,
    },
    Focus(PaneId),
    Exited,
    Sessions(Vec<SessionSummary>),
    Killed,
    NoSuchSession,
    Opened {
        request: u32,
        pane: Option<PaneId>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutableId {
    pub device: u64,
    pub inode: u64,
}
