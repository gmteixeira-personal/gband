use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use gband_protocol::test::{FromProcess, Role, SOCKET_VARIABLE, ToProcess};
use gband_protocol::{HEADER_LEN, MAX_FRAME_LEN, decode, encode};

pub fn requested() -> Option<PathBuf> {
    std::env::var_os(SOCKET_VARIABLE)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
}

pub fn join(path: &PathBuf, role: Role) -> Result<UnixStream> {
    let failed = || format!("cannot join the test channel {}", path.display());
    let mut stream = UnixStream::connect(path).with_context(failed)?;
    let listener = rustix::net::sockopt::socket_peercred(&stream).with_context(failed)?;
    if listener.uid != rustix::process::getuid() {
        bail!(
            "cannot join the test channel {}: it is held by another user",
            path.display()
        );
    }
    let hello = encode(&FromProcess::Hello { role }).with_context(failed)?;
    stream.write_all(&hello).with_context(failed)?;
    let mut header = [0u8; HEADER_LEN];
    stream.read_exact(&mut header).with_context(failed)?;
    let len = u32::from_be_bytes(header) as usize;
    if len > MAX_FRAME_LEN {
        bail!("{}: the runner sent a frame of {len} bytes", failed());
    }
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).with_context(failed)?;
    let ToProcess::Start { time } = decode(&payload).with_context(failed)? else {
        bail!("{}: the runner did not start with a time", failed());
    };
    gband_lua::freeze_time(time);
    Ok(stream)
}
