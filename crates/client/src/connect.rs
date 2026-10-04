use std::ffi::OsStr;
use std::fs;
use std::io::{self, ErrorKind};
use std::os::fd::AsFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use gband_protocol::{
    ClientMessage, Decoder, ExecutableId, Hello, HelloReply, PROTOCOL_VERSION, ServerMessage,
    encode, lock_path,
};
use rustix::process::Uid;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{Instant, sleep};

use crate::ClientConfig;

const RETRY_INTERVAL: Duration = Duration::from_millis(25);
const START_TIMEOUT: Duration = Duration::from_secs(5);
const READ_BUFFER_LEN: usize = 64 * 1024;
const REPLACED_EXECUTABLE_SUFFIX: &[u8] = b" (deleted)";

pub struct Connection {
    pub stream: UnixStream,
    pub decoder: Decoder,
    pub pid: u32,
    pub executable: ExecutableId,
    pub stale_server: bool,
}

impl Connection {
    pub async fn send(&mut self, message: &ClientMessage) -> Result<()> {
        write_frame(&mut self.stream, message).await
    }
}

enum Handshake {
    Ready(Connection),
    Replace(String),
}

pub async fn connect(config: &ClientConfig) -> Result<Connection> {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let mut replaced = false;
    loop {
        let stream = connect_or_start(config).await?;
        match handshake(stream, config, cols, rows, !replaced).await? {
            Handshake::Ready(connection) => return Ok(connection),
            Handshake::Replace(reason) => {
                tracing::info!("replacing the server: {reason}");
                stop_server(&config.executable_path, &config.socket)?;
                replaced = true;
            }
        }
    }
}

async fn handshake(
    mut stream: UnixStream,
    config: &ClientConfig,
    cols: u16,
    rows: u16,
    may_replace: bool,
) -> Result<Handshake> {
    let may_replace = may_replace && config.replace_mismatched && runs_like_the_client(config);
    let mut decoder = Decoder::new();
    write_frame(
        &mut stream,
        &Hello {
            version: PROTOCOL_VERSION,
            cols,
            rows,
        },
    )
    .await?;
    match read_frame(&mut stream, &mut decoder).await? {
        HelloReply::Accepted { .. } => {}
        HelloReply::Rejected { version } if may_replace => {
            return Ok(Handshake::Replace(format!(
                "it speaks protocol version {version}"
            )));
        }
        HelloReply::Rejected { version } => bail!(
            "the server speaks protocol version {version} and this client speaks version \
             {PROTOCOL_VERSION}; stop the server with {}",
            config.kill_command
        ),
    }
    let ServerMessage::Info { pid, executable } = read_frame(&mut stream, &mut decoder).await?
    else {
        bail!("the server did not send its info after the handshake");
    };
    let stale_server = executable != config.identity;
    if stale_server {
        tracing::warn!(pid, "the server runs a different gband build");
        if may_replace {
            let _ = write_frame(&mut stream, &ClientMessage::Detach).await;
            return Ok(Handshake::Replace(format!(
                "process {pid} runs a different build"
            )));
        }
    }
    Ok(Handshake::Ready(Connection {
        stream,
        decoder,
        pid,
        executable,
        stale_server,
    }))
}

async fn connect_or_start(config: &ClientConfig) -> Result<UnixStream> {
    let socket = &config.socket;
    match UnixStream::connect(socket).await {
        Ok(stream) => return owned_by_user(stream, socket),
        Err(error) if is_absent(&error) => {}
        Err(error) => {
            return Err(error).with_context(|| format!("cannot connect to {}", socket.display()));
        }
    }
    tracing::info!("no server on {}, starting one", socket.display());
    let mut child = start_server(&config.executable_path, socket)?;
    let deadline = Instant::now() + START_TIMEOUT;
    let not_started = || {
        format!(
            "the server did not start on {}; see the server log in {}",
            socket.display(),
            config.log_dir.display()
        )
    };
    loop {
        sleep(RETRY_INTERVAL).await;
        match UnixStream::connect(socket).await {
            Ok(stream) => return owned_by_user(stream, socket),
            Err(error) if is_absent(&error) => {}
            Err(error) => return Err(error).with_context(not_started),
        }
        if let Some(status) = child.try_wait().context("cannot wait for the server")? {
            tracing::warn!("the server exited before accepting a connection: {status}");
            bail!(not_started());
        }
        if Instant::now() >= deadline {
            bail!(not_started());
        }
    }
}

fn owned_by_user(stream: UnixStream, socket: &Path) -> Result<UnixStream> {
    check_peer(&stream, socket, rustix::process::getuid())?;
    Ok(stream)
}

fn check_peer(stream: impl AsFd, socket: &Path, uid: Uid) -> Result<()> {
    let peer = rustix::net::sockopt::socket_peercred(stream)
        .with_context(|| format!("cannot read the user serving {}", socket.display()))?
        .uid;
    if peer != uid {
        bail!(
            "{} is served by uid {}, not by uid {}",
            socket.display(),
            peer.as_raw(),
            uid.as_raw()
        );
    }
    Ok(())
}

fn runs_like_the_client(config: &ClientConfig) -> bool {
    server_executable(&config.socket).is_some_and(|server| server == config.executable_path)
}

fn server_executable(socket: &Path) -> Option<PathBuf> {
    let record = fs::read_to_string(lock_path(socket)).ok()?;
    let pid: u32 = record.trim().parse().ok()?;
    let link = fs::read_link(format!("/proc/{pid}/exe")).ok()?;
    let bytes = link.as_os_str().as_bytes();
    let path = bytes
        .strip_suffix(REPLACED_EXECUTABLE_SUFFIX)
        .unwrap_or(bytes);
    Some(PathBuf::from(OsStr::from_bytes(path)))
}

fn is_absent(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        ErrorKind::NotFound | ErrorKind::ConnectionRefused
    )
}

fn start_server(executable: &Path, socket: &Path) -> Result<std::process::Child> {
    let mut command = Command::new(executable);
    command
        .arg("-p")
        .arg(socket)
        .arg("server")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    unsafe {
        command.pre_exec(|| rustix::process::setsid().map(drop).map_err(io::Error::from));
    }
    command
        .spawn()
        .with_context(|| format!("cannot start {} server", executable.display()))
}

fn stop_server(executable: &Path, socket: &Path) -> Result<()> {
    let output = Command::new(executable)
        .arg("-p")
        .arg(socket)
        .arg("kill-server")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("cannot run {} kill-server", executable.display()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let message = stderr.trim().trim_start_matches("gband: ");
        bail!("{message}");
    }
    Ok(())
}

async fn write_frame<T: Serialize>(stream: &mut UnixStream, message: &T) -> Result<()> {
    stream
        .write_all(&encode(message)?)
        .await
        .context("cannot write to the server")
}

async fn read_frame<T: DeserializeOwned>(
    stream: &mut UnixStream,
    decoder: &mut Decoder,
) -> Result<T> {
    let mut buffer = vec![0; READ_BUFFER_LEN];
    loop {
        if let Some(message) = decoder.next_message()? {
            return Ok(message);
        }
        let n = stream
            .read(&mut buffer)
            .await
            .context("cannot read from the server")?;
        if n == 0 {
            bail!("the server closed the connection during the handshake");
        }
        decoder.feed(&buffer[..n])?;
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixStream;

    use rustix::process::getuid;

    use super::*;

    const SOCKET: &str = "/tmp/s.sock";

    #[test]
    fn peer_of_the_same_user_passes() {
        let (stream, _peer) = UnixStream::pair().unwrap();
        check_peer(&stream, Path::new(SOCKET), getuid()).unwrap();
    }

    #[test]
    fn peer_of_another_user_is_refused() {
        let (stream, _peer) = UnixStream::pair().unwrap();
        let other = Uid::from_raw(getuid().as_raw() + 1);
        let error = check_peer(&stream, Path::new(SOCKET), other).unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains(SOCKET), "{message}");
        assert!(
            message.contains(&format!("uid {},", getuid().as_raw())),
            "{message}"
        );
    }
}
