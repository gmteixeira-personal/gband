use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};

use crate::{Config, ConfigError, LoadOptions, Locations, Side, load, user_dir};

const DEBOUNCE: Duration = Duration::from_millis(100);

pub struct Watcher {
    _watcher: Option<Arc<Mutex<RecommendedWatcher>>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Watching {
    Directory,
    Parent,
}

pub fn watch<F>(locations: Locations, side: Side, options: LoadOptions, deliver: F) -> Watcher
where
    F: FnMut(Result<Config, ConfigError>) + Send + 'static,
{
    let idle = Watcher { _watcher: None };
    let directory = user_dir(&locations.config);
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
        if inner.watch(&directory, RecursiveMode::Recursive).is_ok() {
            Watching::Directory
        } else if inner
            .watch(&locations.config, RecursiveMode::NonRecursive)
            .is_ok()
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
        .spawn(move || {
            reload(
                Target {
                    locations,
                    side,
                    options,
                    directory,
                },
                watching,
                events,
                weak,
                deliver,
            )
        });
    if let Err(error) = spawned {
        tracing::warn!("cannot watch the configuration: {error}");
        return idle;
    }
    Watcher {
        _watcher: Some(watcher),
    }
}

struct Target {
    locations: Locations,
    side: Side,
    options: LoadOptions,
    directory: PathBuf,
}

fn reload<F>(
    target: Target,
    mut watching: Watching,
    events: mpsc::Receiver<notify::Result<Event>>,
    watcher: Weak<Mutex<RecommendedWatcher>>,
    mut deliver: F,
) where
    F: FnMut(Result<Config, ConfigError>),
{
    let Target {
        locations,
        side,
        options,
        directory,
    } = target;
    while let Ok(event) = events.recv() {
        let relevant = match watching {
            Watching::Directory => touches_lua(&event, &directory),
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
            if watcher.watch(&directory, RecursiveMode::Recursive).is_ok() {
                let _ = watcher.unwatch(&locations.config);
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
        deliver(load(&locations, side, &options));
    }
}

fn changed(event: &notify::Result<Event>) -> Option<&Event> {
    event
        .as_ref()
        .ok()
        .filter(|event| !matches!(event.kind, EventKind::Access(_)))
}

fn touches(event: &notify::Result<Event>, target: &Path) -> bool {
    changed(event).is_some_and(|event| event.paths.iter().any(|path| path == target))
}

fn touches_lua(event: &notify::Result<Event>, directory: &Path) -> bool {
    changed(event).is_some_and(|event| {
        event.paths.iter().any(|path| {
            path.starts_with(directory)
                && path
                    .file_name()
                    .is_some_and(|name| name.as_encoded_bytes().ends_with(b".lua"))
        })
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::mpsc::Receiver;

    use super::*;
    use crate::{defaults_file, user_file};

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("gband-lua-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn dir(&self) -> PathBuf {
            self.0.join("gband")
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn watched(dir: &Path) -> (Watcher, Receiver<Result<Config, ConfigError>>) {
        let (tx, rx) = mpsc::channel();
        let locations = Locations {
            config: dir.to_path_buf(),
            plugins: None,
        };
        let watcher = watch(
            locations,
            Side::Client,
            LoadOptions::default(),
            move |result| {
                let _ = tx.send(result);
            },
        );
        (watcher, rx)
    }

    fn next(rx: &Receiver<Result<Config, ConfigError>>) -> Config {
        rx.recv_timeout(Duration::from_secs(1))
            .expect("a reload within one second")
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
            "gband.bind('alt+k', gband.action.focus_pane_up)",
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
        assert!(rx.recv_timeout(Duration::from_secs(1)).is_err());
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
        assert!(rx.recv_timeout(Duration::from_secs(1)).is_err());
    }

    #[test]
    fn missing_user_directory_is_watched_through_the_configuration_directory() {
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
}
