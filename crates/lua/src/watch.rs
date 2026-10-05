use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};

use crate::{Config, ConfigError, load};

const DEBOUNCE: Duration = Duration::from_millis(100);

pub fn config_path() -> Option<PathBuf> {
    config_path_from(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

pub fn config_path_from(
    xdg_config_home: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    let base = match xdg_config_home
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        Some(base) => base,
        None => PathBuf::from(home.filter(|home| !home.is_empty())?).join(".config"),
    };
    Some(base.join("gband").join("init.lua"))
}

pub struct Watcher {
    _watcher: Option<Arc<Mutex<RecommendedWatcher>>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Watching {
    Directory,
    Parent,
}

pub fn watch<F>(path: PathBuf, deliver: F) -> Watcher
where
    F: FnMut(Result<Config, ConfigError>) + Send + 'static,
{
    let idle = Watcher { _watcher: None };
    let Some(directory) = path.parent().map(Path::to_path_buf) else {
        return idle;
    };
    let (events_tx, events) = mpsc::channel();
    let watcher = match notify::recommended_watcher(move |event| {
        let _ = events_tx.send(event);
    }) {
        Ok(watcher) => Arc::new(Mutex::new(watcher)),
        Err(error) => {
            tracing::warn!("cannot watch the configuration: {error}");
            return idle;
        }
    };
    let watching = {
        let mut inner = watcher.lock().expect("watcher lock");
        if inner.watch(&directory, RecursiveMode::NonRecursive).is_ok() {
            Watching::Directory
        } else if directory
            .parent()
            .is_some_and(|parent| inner.watch(parent, RecursiveMode::NonRecursive).is_ok())
        {
            Watching::Parent
        } else {
            tracing::info!(
                "not watching the configuration: {} does not exist",
                directory.display()
            );
            return idle;
        }
    };
    let weak = Arc::downgrade(&watcher);
    let spawned = thread::Builder::new()
        .name("gband-config".to_owned())
        .spawn(move || reload(path, directory, watching, events, weak, deliver));
    if let Err(error) = spawned {
        tracing::warn!("cannot watch the configuration: {error}");
        return idle;
    }
    Watcher {
        _watcher: Some(watcher),
    }
}

fn reload<F>(
    path: PathBuf,
    directory: PathBuf,
    mut watching: Watching,
    events: mpsc::Receiver<notify::Result<Event>>,
    watcher: Weak<Mutex<RecommendedWatcher>>,
    mut deliver: F,
) where
    F: FnMut(Result<Config, ConfigError>),
{
    while let Ok(event) = events.recv() {
        let relevant = match watching {
            Watching::Directory => touches(&event, &path),
            Watching::Parent => touches(&event, &directory),
        };
        if !relevant {
            continue;
        }
        if watching == Watching::Parent && directory.is_dir() {
            let Some(watcher) = watcher.upgrade() else {
                return;
            };
            let mut watcher = watcher.lock().expect("watcher lock");
            if watcher
                .watch(&directory, RecursiveMode::NonRecursive)
                .is_ok()
            {
                if let Some(parent) = directory.parent() {
                    let _ = watcher.unwatch(parent);
                }
                watching = Watching::Directory;
            }
        }
        let deadline = Instant::now() + DEBOUNCE;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match events.recv_timeout(left) {
                Ok(_) => {}
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        deliver(load(&path));
    }
}

fn touches(event: &notify::Result<Event>, target: &Path) -> bool {
    let Ok(event) = event else {
        return false;
    };
    !matches!(event.kind, EventKind::Access(_)) && event.paths.iter().any(|path| path == target)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::mpsc::Receiver;

    use super::*;

    fn os(text: &str) -> Option<OsString> {
        Some(OsString::from(text))
    }

    #[test]
    fn xdg_config_home_wins_when_absolute() {
        assert_eq!(
            config_path_from(os("/tmp/cfg"), os("/home/u")),
            Some(PathBuf::from("/tmp/cfg/gband/init.lua"))
        );
    }

    #[test]
    fn home_is_the_fallback() {
        let expected = Some(PathBuf::from("/home/u/.config/gband/init.lua"));
        assert_eq!(config_path_from(None, os("/home/u")), expected);
        assert_eq!(config_path_from(os("relative"), os("/home/u")), expected);
        assert_eq!(config_path_from(os(""), os("/home/u")), expected);
        assert_eq!(config_path_from(None, None), None);
    }

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("gband-lua-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn watched(path: &Path) -> (Watcher, Receiver<Result<Config, ConfigError>>) {
        let (tx, rx) = mpsc::channel();
        let watcher = watch(path.to_path_buf(), move |result| {
            let _ = tx.send(result);
        });
        (watcher, rx)
    }

    fn next(rx: &Receiver<Result<Config, ConfigError>>) -> Config {
        rx.recv_timeout(Duration::from_secs(1))
            .expect("a reload within one second")
            .expect("a good configuration")
    }

    fn prefix_of(config: &Config) -> String {
        format!("{:?}", config.options.prefix)
    }

    #[test]
    fn write_rename_and_delete_each_reload() {
        let scratch = Scratch::new("watch");
        let path = scratch.0.join("gband").join("init.lua");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let (_watcher, rx) = watched(&path);
        let defaults = prefix_of(&crate::defaults());

        fs::write(&path, "gband.set { prefix = 'ctrl+b' }").unwrap();
        let written = next(&rx);
        assert_ne!(prefix_of(&written), defaults);
        while rx.recv_timeout(Duration::from_millis(300)).is_ok() {}

        let temporary = path.with_file_name("init.lua.tmp");
        fs::write(&temporary, "gband.set { prefix = 'ctrl+x' }").unwrap();
        fs::rename(&temporary, &path).unwrap();
        let renamed = next(&rx);
        assert!(prefix_of(&renamed).contains("'x'"));
        while rx.recv_timeout(Duration::from_millis(300)).is_ok() {}

        fs::remove_file(&path).unwrap();
        assert_eq!(prefix_of(&next(&rx)), defaults);
    }

    #[test]
    fn missing_directory_is_watched_through_its_parent() {
        let scratch = Scratch::new("parent");
        let path = scratch.0.join("gband").join("init.lua");
        let (_watcher, rx) = watched(&path);
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
}
