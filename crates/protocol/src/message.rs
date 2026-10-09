use gband_core::input::{Key, MouseEvent};
use gband_core::layout::{Layout, SessionAction, WindowId};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::session::{SessionName, SessionSummary};
use crate::value::Value;

pub const PROTOCOL_VERSION: u32 = 12;

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
    Mouse {
        window: WindowId,
        event: MouseEvent,
    },
    Rename {
        window: WindowId,
        name: Option<String>,
    },
    Control {
        session: SessionName,
        target: Target,
        operation: Operation,
    },
    Reload,
    ControlAnswer {
        call: u64,
        answer: Answer,
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
    WindowName {
        window: WindowId,
        automatic: String,
        manual: Option<String>,
    },
    Result {
        call: u64,
        result: Result<Value, String>,
    },
    Requirements(Vec<Requirement>),
    ServerError(String),
    Reloaded,
    Control {
        call: u64,
        operation: Operation,
    },
    ControlResults(Vec<Entry>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Server,
    EveryClient,
    All,
    Client(u64),
    Chosen,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    Reload,
    Errors,
    Eval { source: String, args: Vec<Value> },
    Command { name: String, args: Value },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Answer {
    Loaded { load: u64, error: Option<String> },
    Errors { load: u64, errors: Vec<String> },
    Values(Result<Vec<Value>, String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Process {
    Server,
    Client(u64),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub process: Process,
    pub answer: Option<Answer>,
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
