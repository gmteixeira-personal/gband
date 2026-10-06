use gband_core::input::Key;
use gband_core::layout::{Layout, SessionAction, WindowId};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::session::{SessionName, SessionSummary};
use crate::value::Value;

pub const PROTOCOL_VERSION: u32 = 8;

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
    Attach {
        session: SessionName,
        cwd: PathBuf,
    },
    ListSessions,
    KillSession {
        session: SessionName,
    },
    Key {
        window: WindowId,
        key: Key,
    },
    Paste {
        window: WindowId,
        text: String,
    },
    Resize {
        cols: u16,
        rows: u16,
    },
    Action(SessionAction),
    Detach,
    Shown(Vec<WindowId>),
    Content {
        window: WindowId,
        output: Vec<u8>,
    },
    Command {
        call: u64,
        name: String,
        args: Value,
    },
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
        window: WindowId,
        cols: u16,
        rows: u16,
        contents: Vec<u8>,
    },
    Update {
        window: WindowId,
        contents: Vec<u8>,
    },
    Focus(WindowId),
    Exited,
    Sessions(Vec<SessionSummary>),
    Killed,
    NoSuchSession,
    Opened {
        request: u32,
        window: Option<WindowId>,
    },
    Event {
        name: String,
        data: Value,
        queued: bool,
        time: u64,
    },
    WindowState {
        window: WindowId,
        key: String,
        value: Option<Value>,
    },
    Result {
        call: u64,
        result: Result<Value, String>,
    },
    Requirements(Vec<Requirement>),
    ServerError(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    pub plugin: String,
    pub requirement: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutableId {
    pub device: u64,
    pub inode: u64,
}
