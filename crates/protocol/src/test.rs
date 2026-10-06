use serde::{Deserialize, Serialize};

use crate::value::Value;

pub const SOCKET_VARIABLE: &str = "GBAND_TEST_SOCKET";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Client,
    Server,
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Client => "client",
            Role::Server => "server",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ToProcess {
    Start {
        time: Option<i64>,
    },
    Eval {
        id: u64,
        source: String,
        args: Vec<Value>,
    },
    Settle {
        round: u32,
        input: Option<u64>,
    },
    Reload {
        id: u64,
    },
    SetTime {
        time: Option<i64>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FromProcess {
    Hello {
        role: Role,
    },
    Answer {
        id: u64,
        result: Result<Vec<Value>, String>,
    },
    Settled {
        round: u32,
        sent: u64,
    },
    Reloaded {
        id: u64,
        error: Option<String>,
    },
}
