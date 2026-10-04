use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use gband_protocol::socket_path;
use rustix::process::{Pid, Signal};

pub const LOCK_NAME: &str = "default.lock";

const POLL_INTERVAL: Duration = Duration::from_millis(25);
const EMPTY_RECORD_GRACE: Duration = Duration::from_secs(1);
const STOP_TIMEOUT: Duration = Duration::from_secs(5);

pub fn lock_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join(LOCK_NAME)
}

pub fn acquire(runtime_dir: &Path) -> Result<File> {
    let path = lock_path(runtime_dir);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .with_context(|| format!("cannot open {}", path.display()))?;
    match file.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => bail!(
            "a server is already running on {}",
            socket_path(runtime_dir).display()
        ),
        Err(TryLockError::Error(error)) => {
            return Err(error).with_context(|| format!("cannot lock {}", path.display()));
        }
    }
    file.set_len(0)
        .and_then(|()| writeln!(file, "{}", std::process::id()))
        .with_context(|| format!("cannot record the server pid in {}", path.display()))?;
    Ok(file)
}

pub fn kill(runtime_dir: &Path) -> Result<()> {
    let not_running = || {
        anyhow!(
            "no server is running on {}",
            socket_path(runtime_dir).display()
        )
    };
    let path = lock_path(runtime_dir);
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Err(not_running()),
        Err(error) => return Err(error).with_context(|| format!("cannot open {}", path.display())),
    };
    if is_free(&file, &path)? {
        return Err(not_running());
    }
    let pid = read_pid(&path)?;
    tracing::info!(pid = pid.as_raw_pid(), "stopping the server");
    rustix::process::kill_process(pid, Signal::TERM)
        .with_context(|| format!("cannot signal server process {}", pid.as_raw_pid()))?;
    let deadline = Instant::now() + STOP_TIMEOUT;
    while Instant::now() < deadline {
        thread::sleep(POLL_INTERVAL);
        if is_free(&file, &path)? {
            tracing::info!(pid = pid.as_raw_pid(), "server stopped");
            return Ok(());
        }
    }
    bail!(
        "server process {} did not stop within {} seconds",
        pid.as_raw_pid(),
        STOP_TIMEOUT.as_secs()
    )
}

fn is_free(file: &File, path: &Path) -> Result<bool> {
    match file.try_lock() {
        Ok(()) => {
            file.unlock()
                .with_context(|| format!("cannot unlock {}", path.display()))?;
            Ok(true)
        }
        Err(TryLockError::WouldBlock) => Ok(false),
        Err(TryLockError::Error(error)) => {
            Err(error).with_context(|| format!("cannot test the lock on {}", path.display()))
        }
    }
}

fn read_pid(path: &Path) -> Result<Pid> {
    let deadline = Instant::now() + EMPTY_RECORD_GRACE;
    loop {
        let record =
            fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
        let record = record.trim();
        if !record.is_empty() {
            return record
                .parse()
                .ok()
                .and_then(Pid::from_raw)
                .with_context(|| format!("{} holds no valid pid: {record:?}", path.display()));
        }
        if Instant::now() >= deadline {
            bail!("{} holds no pid", path.display());
        }
        thread::sleep(POLL_INTERVAL);
    }
}
