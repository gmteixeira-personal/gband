use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::user_dir;

const POLL: Duration = Duration::from_secs(1);
const QUIET: Duration = Duration::from_millis(100);

pub struct Watcher {
    _stop: Option<Sender<()>>,
}

#[derive(PartialEq, Eq)]
struct Stamp {
    modified: Option<SystemTime>,
    len: u64,
    target: Option<PathBuf>,
}

type Snapshot = BTreeMap<PathBuf, Stamp>;

pub fn watch<F>(config: PathBuf, changed: F) -> Watcher
where
    F: FnMut() + Send + 'static,
{
    let directory = user_dir(&config);
    let seen = snapshot(&directory);
    let (stop_tx, stop) = mpsc::channel();
    let spawned = thread::Builder::new()
        .name("gband-config".to_owned())
        .spawn(move || poll(&directory, seen, &stop, changed));
    match spawned {
        Ok(_) => Watcher {
            _stop: Some(stop_tx),
        },
        Err(error) => {
            tracing::warn!("cannot watch the configuration: {error}");
            Watcher { _stop: None }
        }
    }
}

fn poll<F>(directory: &Path, mut seen: Snapshot, stop: &Receiver<()>, mut changed: F)
where
    F: FnMut(),
{
    while wait(stop, POLL) {
        let mut current = snapshot(directory);
        if current == seen {
            continue;
        }
        loop {
            if !wait(stop, QUIET) {
                return;
            }
            let settled = snapshot(directory);
            if settled == current {
                break;
            }
            current = settled;
        }
        seen = current;
        changed();
    }
}

fn wait(stop: &Receiver<()>, period: Duration) -> bool {
    matches!(stop.recv_timeout(period), Err(RecvTimeoutError::Timeout))
}

fn snapshot(directory: &Path) -> Snapshot {
    let mut files = Snapshot::new();
    scan(directory, &mut HashSet::new(), &mut files);
    files
}

fn scan(directory: &Path, followed: &mut HashSet<PathBuf>, files: &mut Snapshot) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if kind.is_dir() {
            scan(&path, followed, files);
        } else if kind.is_symlink() {
            linked(path, followed, files);
        } else if kind.is_file()
            && is_lua(&path)
            && let Ok(metadata) = entry.metadata()
        {
            files.insert(path, stamp(&metadata, None));
        }
    }
}

fn linked(path: PathBuf, followed: &mut HashSet<PathBuf>, files: &mut Snapshot) {
    let (Ok(metadata), Ok(target)) = (fs::metadata(&path), fs::canonicalize(&path)) else {
        return;
    };
    if metadata.is_dir() {
        if followed.insert(target) {
            scan(&path, followed, files);
        }
    } else if is_lua(&path) {
        files.insert(path, stamp(&metadata, Some(target)));
    }
}

fn stamp(metadata: &fs::Metadata, target: Option<PathBuf>) -> Stamp {
    Stamp {
        modified: metadata.modified().ok(),
        len: metadata.len(),
        target,
    }
}

fn is_lua(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.as_encoded_bytes().ends_with(b".lua"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::mpsc::Receiver;

    use super::*;
    use crate::{
        Config, ConfigError, LoadOptions, Locations, Side, defaults_file, load, user_file,
    };

    struct Scratch(gband_scratch::Scratch);

    impl Scratch {
        fn new(name: &str) -> Self {
            Self(gband_scratch::Scratch::new("lua", name))
        }

        fn dir(&self) -> PathBuf {
            self.0.join("gband")
        }
    }

    fn watched(dir: &Path) -> (Watcher, Receiver<Result<Config, ConfigError>>) {
        let (tx, rx) = mpsc::channel();
        let locations = Locations {
            config: dir.to_path_buf(),
            plugins: None,
        };
        let watcher = watch(locations.config.clone(), move || {
            let _ = tx.send(load(&locations, Side::Client, &LoadOptions::default()));
        });
        (watcher, rx)
    }

    fn next(rx: &Receiver<Result<Config, ConfigError>>) -> Config {
        rx.recv_timeout(Duration::from_secs(2))
            .expect("a reload within two seconds")
            .expect("a good configuration")
    }

    fn settle(rx: &Receiver<Result<Config, ConfigError>>) {
        while rx.recv_timeout(Duration::from_millis(300)).is_ok() {}
    }

    fn prefix_of(config: &Config) -> String {
        format!("{:?}", config.options.prefix)
    }

    #[test]
    fn write_rename_and_delete_each_reload() {
        let scratch = Scratch::new("watch");
        let dir = scratch.dir();
        crate::prepare(&dir).unwrap();
        let path = user_file(&dir, Side::Client);
        let (_watcher, rx) = watched(&dir);
        let defaults = prefix_of(&crate::defaults(Side::Client));

        fs::write(&path, "gband.set { prefix = 'ctrl+b' }").unwrap();
        let written = next(&rx);
        assert_ne!(prefix_of(&written), defaults);
        settle(&rx);

        let temporary = path.with_file_name("init.lua.tmp");
        fs::write(&temporary, "gband.set { prefix = 'ctrl+x' }").unwrap();
        fs::rename(&temporary, &path).unwrap();
        let renamed = next(&rx);
        assert!(prefix_of(&renamed).contains("'x'"));
        settle(&rx);

        fs::remove_file(&path).unwrap();
        assert_eq!(prefix_of(&next(&rx)), defaults);
    }

    #[test]
    fn successive_writes_each_reload() {
        let scratch = Scratch::new("quick");
        let dir = scratch.dir();
        crate::prepare(&dir).unwrap();
        let path = user_file(&dir, Side::Client);
        let (_watcher, rx) = watched(&dir);

        fs::write(&path, "gband.set { prefix = 'ctrl+b' }").unwrap();
        assert!(prefix_of(&next(&rx)).contains("'b'"));

        fs::write(&path, "gband.set { prefix = 'ctrl+x' }").unwrap();
        assert!(prefix_of(&next(&rx)).contains("'x'"));
    }

    #[test]
    fn user_plugin_file_edited() {
        let scratch = Scratch::new("user-plugin");
        let dir = scratch.dir();
        crate::prepare(&dir).unwrap();
        let modules = dir.join("user").join("lua");
        fs::create_dir_all(&modules).unwrap();
        fs::write(modules.join("keys.lua"), "").unwrap();
        fs::write(user_file(&dir, Side::Client), "require('keys')").unwrap();
        let (_watcher, rx) = watched(&dir);
        fs::write(
            modules.join("keys.lua"),
            "gband.bind('alt+k', gband.action.focus_window_up)",
        )
        .unwrap();
        let config = loop {
            let config = next(&rx);
            if config.keymap.contains_key("root") {
                break config;
            }
        };
        assert_eq!(config.keymap["root"].len(), 1);
        settle(&rx);
        fs::write(modules.join("notes.txt"), "notes").unwrap();
        assert!(rx.recv_timeout(Duration::from_secs(2)).is_err());
    }

    #[test]
    fn defaults_file_is_not_watched() {
        let scratch = Scratch::new("defaults");
        let dir = scratch.dir();
        crate::prepare(&dir).unwrap();
        let (_watcher, rx) = watched(&dir);
        fs::write(
            defaults_file(&dir, Side::Client),
            "gband.set { prefix = 'ctrl+b' }",
        )
        .unwrap();
        assert!(rx.recv_timeout(Duration::from_secs(2)).is_err());
    }

    #[test]
    fn missing_user_directory_is_watched_once_created() {
        let scratch = Scratch::new("parent");
        let dir = scratch.dir();
        fs::create_dir_all(&dir).unwrap();
        let path = user_file(&dir, Side::Client);
        let (_watcher, rx) = watched(&dir);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let _ = rx.recv_timeout(Duration::from_millis(300));
        fs::write(&path, "gband.set { prefix = 'ctrl+b' }").unwrap();
        let config = loop {
            let config = next(&rx);
            if prefix_of(&config).contains("'b'") {
                break config;
            }
        };
        assert!(prefix_of(&config).contains("'b'"));
    }

    #[test]
    fn symlinks_are_followed_without_looping() {
        let scratch = Scratch::new("symlink");
        let dir = scratch.dir();
        crate::prepare(&dir).unwrap();
        let user = dir.join("user");
        let elsewhere = scratch.0.join("elsewhere");
        fs::create_dir_all(&elsewhere).unwrap();
        std::os::unix::fs::symlink(&user, user.join("loop")).unwrap();
        std::os::unix::fs::symlink(&elsewhere, user.join("lua")).unwrap();
        fs::write(user_file(&dir, Side::Client), "require('keys')").unwrap();
        fs::write(elsewhere.join("keys.lua"), "").unwrap();
        let (_watcher, rx) = watched(&dir);
        fs::write(
            elsewhere.join("keys.lua"),
            "gband.bind('alt+k', gband.action.focus_window_up)",
        )
        .unwrap();
        let config = next(&rx);
        assert_eq!(config.keymap["root"].len(), 1);
    }
}
