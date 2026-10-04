mod frame;
mod io;
mod message;
mod session;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub use frame::{Decoder, FrameError, HEADER_LEN, MAX_FRAME_LEN, decode, encode, leading_version};
pub use io::{IoError, MessageReader, MessageWriter};
pub use message::{
    ClientMessage, ExecutableId, Hello, HelloReply, PROTOCOL_VERSION, ServerMessage,
};
pub use session::{DEFAULT_SESSION, SESSION_NAME_MAX, SessionName, SessionSummary};

pub const SOCKET_NAME: &str = "default.sock";

pub fn socket_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join(SOCKET_NAME)
}

pub fn lock_path(socket: &Path) -> PathBuf {
    if socket
        .extension()
        .is_some_and(|extension| extension == "sock")
    {
        return socket.with_extension("lock");
    }
    let mut path = OsString::from(socket);
    path.push(".lock");
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_replaces_a_sock_extension() {
        assert_eq!(
            lock_path(Path::new("default.sock")),
            Path::new("default.lock")
        );
        assert_eq!(lock_path(Path::new("x.sock")), Path::new("x.lock"));
    }

    #[test]
    fn lock_is_appended_to_any_other_path() {
        assert_eq!(lock_path(Path::new("/tmp/s")), Path::new("/tmp/s.lock"));
        assert_eq!(
            lock_path(Path::new("/tmp/s.socket")),
            Path::new("/tmp/s.socket.lock")
        );
    }
}
