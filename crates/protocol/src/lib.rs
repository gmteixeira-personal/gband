mod frame;
mod message;

use std::path::{Path, PathBuf};

pub use frame::{Decoder, FrameError, HEADER_LEN, MAX_FRAME_LEN, encode};
pub use message::{
    ClientMessage, ExecutableId, Hello, HelloReply, PROTOCOL_VERSION, ServerMessage,
};

pub const SOCKET_NAME: &str = "default.sock";

pub fn socket_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join(SOCKET_NAME)
}
