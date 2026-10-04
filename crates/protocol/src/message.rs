use gband_core::input::Key;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

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
    Key(Key),
    Paste(String),
    Resize { cols: u16, rows: u16 },
    Detach,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMessage {
    Info {
        pid: u32,
        executable: ExecutableId,
    },
    Snapshot {
        cols: u16,
        rows: u16,
        contents: Vec<u8>,
    },
    Update(Vec<u8>),
    Exited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutableId {
    pub device: u64,
    pub inode: u64,
}
