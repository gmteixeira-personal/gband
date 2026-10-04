use gband_core::input::Key;
use gband_core::layout::{Layout, PaneId, SessionAction};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 2;

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
    Key { pane: PaneId, key: Key },
    Paste { pane: PaneId, text: String },
    Resize { cols: u16, rows: u16 },
    Action(SessionAction),
    Detach,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutableId {
    pub device: u64,
    pub inode: u64,
}
