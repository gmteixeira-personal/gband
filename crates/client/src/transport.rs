use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io::{self, ErrorKind};
use std::os::fd::AsFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use gband_core::geometry::Size;
use gband_protocol::{SessionName, lock_path};
use rustix::process::Uid;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::UnixStream;
use tokio::time::{Instant, sleep};

const RETRY_INTERVAL: Duration = Duration::from_millis(25);
const START_TIMEOUT: Duration = Duration::from_secs(5);
const REPLACED_EXECUTABLE_SUFFIX: &[u8] = b" (deleted)";

pub struct Link {
    pub reader: Box<dyn AsyncRead + Send + Unpin>,
    pub writer: Box<dyn AsyncWrite + Send + Unpin>,
}

pub trait Transport: fmt::Display {
    fn open(&self, size: Size) -> impl Future<Output = Result<Link>> + Send;
    fn may_replace_server(&self) -> bool;
    fn replace_server(&self) -> impl Future<Output = Result<()>> + Send;
}

pub struct UnixTransport {
    pub socket: PathBuf,
    pub executable_path: PathBuf,
    pub session: SessionName,
    pub start_server: bool,
    pub log_dir: PathBuf,
}

impl fmt::Display for UnixTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.socket.display().fmt(f)
    }
}

impl Transport for UnixTransport {
    async fn open(&self, size: Size) -> Result<Link> {
        let (reader, writer) = self.connect_or_start(size).await?.into_split();
        Ok(Link {
            reader: Box::new(reader),
            writer: Box::new(writer),
        })
    }

    fn may_replace_server(&self) -> bool {
        server_executable(&self.socket).is_some_and(|server| server == self.executable_path)
    }

    async fn replace_server(&self) -> Result<()> {
        stop_server(&self.executable_path, &self.socket)
    }
}

impl UnixTransport {
    async fn connect_or_start(&self, size: Size) -> Result<UnixStream> {
        let socket = &self.socket;
        match UnixStream::connect(socket).await {
            Ok(stream) => return owned_by_user(stream, socket),
            Err(error) if is_absent(&error) && self.start_server => {}
            Err(error) if is_absent(&error) => {
                bail!("no server is running on {}", socket.display())
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("cannot connect to {}", socket.display()));
            }
        }
        tracing::info!("no server on {}, starting one", socket.display());
        let mut child = start_server(&self.executable_path, socket, &self.session, size)?;
        let deadline = Instant::now() + START_TIMEOUT;
        let not_started = || {
            format!(
                "the server did not start on {}; see the server log in {}",
                socket.display(),
                self.log_dir.display()
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

fn start_server(
    executable: &Path,
    socket: &Path,
    session: &SessionName,
    size: Size,
) -> Result<std::process::Child> {
    let mut command = Command::new(executable);
    command
        .arg("-p")
        .arg(socket)
        .arg("server")
        .arg("-s")
        .arg(session.as_str())
        .arg("--size")
        .arg(format!("{}x{}", size.cols, size.rows))
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
