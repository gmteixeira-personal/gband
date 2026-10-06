use std::fs;
use std::ops::Deref;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};

use rustix::io::Errno;
use rustix::process::{Pid, test_kill_process};

#[derive(Debug)]
pub struct Scratch {
    path: PathBuf,
    stem: String,
}

impl Scratch {
    pub fn new(kind: &str, name: &str) -> Self {
        let stem = format!("gband-{kind}-{name}-");
        #[allow(clippy::disallowed_methods)]
        let path = std::env::temp_dir().join(format!("{stem}{}", std::process::id()));
        remove(&path);
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", path.display()));
        Self { path, stem }
    }

    fn sweep(&self) {
        let Some(parent) = self.path.parent() else {
            return;
        };
        let Ok(entries) = fs::read_dir(parent) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(suffix) = name.to_str().and_then(|name| name.strip_prefix(&self.stem)) else {
                continue;
            };
            if !suffix.is_empty()
                && suffix.bytes().all(|byte| byte.is_ascii_digit())
                && suffix
                    .parse()
                    .ok()
                    .and_then(Pid::from_raw)
                    .is_some_and(is_dead)
            {
                remove(&entry.path());
            }
        }
    }
}

impl Deref for Scratch {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for Scratch {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("kept {}", self.path.display());
            return;
        }
        remove(&self.path);
        self.sweep();
    }
}

fn is_dead(pid: Pid) -> bool {
    test_kill_process(pid) == Err(Errno::SRCH)
}

fn remove(path: &Path) {
    if fs::symlink_metadata(path).is_err() || fs::remove_dir_all(path).is_ok() {
        return;
    }
    unlock(path);
    let _ = fs::remove_dir_all(path);
}

fn unlock(path: &Path) {
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o700));
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            unlock(&entry.path());
        }
    }
}
