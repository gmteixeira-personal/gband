use std::ffi::OsString;
use std::fmt;
use std::process::{ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use gband_core::geometry::Size;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStderr, Command};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::Instrument;

use crate::transport::{Link, Transport};

const FINISH_TIMEOUT: Duration = Duration::from_secs(2);
const STDERR_TIMEOUT: Duration = Duration::from_secs(1);
const NOT_FOUND: i32 = 127;

pub type RemoteCommand = Box<dyn Fn(Size) -> Vec<OsString> + Send + Sync>;

pub struct SshTransport {
    destination: OsString,
    gband: String,
    remote_command: RemoteCommand,
    child: tokio::sync::Mutex<Option<Spawned>>,
    last_line: Arc<Mutex<Option<String>>>,
}

struct Spawned {
    child: Child,
    stderr: JoinHandle<()>,
}

impl SshTransport {
    pub fn new(destination: OsString, gband: String, remote_command: RemoteCommand) -> Self {
        Self {
            destination,
            gband,
            remote_command,
            child: tokio::sync::Mutex::new(None),
            last_line: Arc::new(Mutex::new(None)),
        }
    }
}

impl fmt::Display for SshTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.destination.display().fmt(f)
    }
}

impl Transport for SshTransport {
    async fn open(&self, size: Size) -> Result<Link> {
        let mut child = Command::new("ssh")
            .arg("-T")
            .arg("--")
            .arg(&self.destination)
            .args((self.remote_command)(size))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("cannot run ssh")?;
        let (Some(writer), Some(reader), Some(stderr)) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take())
        else {
            bail!("ssh started without its standard streams");
        };
        let stderr =
            tokio::spawn(log_stderr(stderr, Arc::clone(&self.last_line)).in_current_span());
        *self.child.lock().await = Some(Spawned { child, stderr });
        Ok(Link {
            reader: Box::new(reader),
            writer: Box::new(writer),
        })
    }

    fn may_replace_server(&self) -> bool {
        false
    }

    async fn replace_server(&self) -> Result<()> {
        bail!("a server on {self} cannot be replaced")
    }

    fn remote(&self) -> bool {
        true
    }

    async fn diagnose(&self) -> Option<anyhow::Error> {
        let mut spawned = self.child.lock().await;
        let spawned = spawned.as_mut()?;
        let status = spawned.child.wait().await.ok()?;
        let _ = timeout(STDERR_TIMEOUT, &mut spawned.stderr).await;
        let last_line = self.last_line.lock().ok()?.take();
        Some(diagnosis(&self.destination, &self.gband, status, last_line))
    }

    async fn finish(&self) {
        let Some(mut spawned) = self.child.lock().await.take() else {
            return;
        };
        if timeout(FINISH_TIMEOUT, spawned.child.wait()).await.is_err() {
            tracing::warn!("ssh did not exit within {FINISH_TIMEOUT:?}, killing it");
            let _ = spawned.child.kill().await;
        }
    }
}

async fn log_stderr(stderr: ChildStderr, last_line: Arc<Mutex<Option<String>>>) {
    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        tracing::warn!("ssh: {line}");
        if let Ok(mut last) = last_line.lock() {
            *last = Some(line);
        }
    }
}

fn diagnosis(
    destination: &OsString,
    gband: &str,
    status: ExitStatus,
    last_line: Option<String>,
) -> anyhow::Error {
    let destination = destination.display();
    if status.code() == Some(NOT_FOUND) {
        return anyhow!(
            "{gband} was not found on {destination}; install gband there or pass --remote-gband"
        );
    }
    let reason = last_line.unwrap_or_else(|| match status.code() {
        Some(code) => format!("ssh exited with status {code}"),
        None => format!("ssh ended: {status}"),
    });
    anyhow!("cannot reach {destination}: {reason}")
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::ExitStatusExt;

    use super::*;

    fn exited(code: i32) -> ExitStatus {
        ExitStatus::from_raw(code << 8)
    }

    #[test]
    fn missing_gband_names_the_command_and_the_destination() {
        let error = diagnosis(
            &"u@h".into(),
            "gband",
            exited(127),
            Some("sh: gband".into()),
        );
        assert_eq!(
            error.to_string(),
            "gband was not found on u@h; install gband there or pass --remote-gband"
        );
    }

    #[test]
    fn unreachable_host_carries_the_last_line_from_ssh() {
        let line = "ssh: Could not resolve hostname nowhere: Name or service not known";
        let error = diagnosis(&"nowhere".into(), "gband", exited(255), Some(line.into()));
        assert_eq!(error.to_string(), format!("cannot reach nowhere: {line}"));
    }

    #[test]
    fn silent_ssh_names_its_exit_status() {
        let error = diagnosis(&"u@h".into(), "gband", exited(255), None);
        assert_eq!(
            error.to_string(),
            "cannot reach u@h: ssh exited with status 255"
        );
    }
}
