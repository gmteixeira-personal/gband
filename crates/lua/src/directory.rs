use std::ffi::OsString;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::DEFAULTS;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locations {
    pub config: PathBuf,
    pub plugins: Option<PathBuf>,
}

impl Locations {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            config: config_dir()?,
            plugins: plugins_dir(),
        })
    }
}

pub fn config_dir() -> Option<PathBuf> {
    config_dir_from(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

pub fn config_dir_from(
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
    Some(base.join("gband"))
}

pub fn plugins_dir() -> Option<PathBuf> {
    plugins_dir_from(std::env::var_os("XDG_DATA_HOME"), std::env::var_os("HOME"))
}

pub fn plugins_dir_from(
    xdg_data_home: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    let base = match xdg_data_home
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        Some(base) => base,
        None => PathBuf::from(home.filter(|home| !home.is_empty())?)
            .join(".local")
            .join("share"),
    };
    Some(base.join("gband").join("plugins"))
}

pub fn user_dir(dir: &Path) -> PathBuf {
    dir.join("user")
}

pub fn user_file(dir: &Path) -> PathBuf {
    user_dir(dir).join("init.lua")
}

pub fn defaults_file(dir: &Path) -> PathBuf {
    dir.join("defaults").join("init.lua")
}

pub fn prepare(dir: &Path) -> io::Result<()> {
    let defaults = defaults_file(dir);
    let defaults_dir = defaults.parent().expect("defaults_file has a parent");
    fs::create_dir_all(defaults_dir)?;
    fs::create_dir_all(dir.join("user"))?;
    match fs::read(&defaults) {
        Ok(current) if current == DEFAULTS.as_bytes() => return Ok(()),
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    static WRITES: AtomicU64 = AtomicU64::new(0);
    let temporary = defaults_dir.join(format!(
        ".init.lua.{}.{}",
        std::process::id(),
        WRITES.fetch_add(1, Ordering::Relaxed)
    ));
    let written = fs::write(&temporary, DEFAULTS).and_then(|()| fs::rename(&temporary, &defaults));
    if written.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    written
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::time::Duration;

    use super::*;

    fn os(text: &str) -> Option<OsString> {
        Some(OsString::from(text))
    }

    #[test]
    fn xdg_config_home_wins_when_absolute() {
        assert_eq!(
            config_dir_from(os("/tmp/cfg"), os("/home/u")),
            Some(PathBuf::from("/tmp/cfg/gband"))
        );
    }

    #[test]
    fn home_is_the_fallback() {
        let expected = Some(PathBuf::from("/home/u/.config/gband"));
        assert_eq!(config_dir_from(None, os("/home/u")), expected);
        assert_eq!(config_dir_from(os("relative"), os("/home/u")), expected);
        assert_eq!(config_dir_from(os(""), os("/home/u")), expected);
        assert_eq!(config_dir_from(None, None), None);
        assert_eq!(config_dir_from(os("relative"), None), None);
    }

    #[test]
    fn xdg_data_home_wins_when_absolute() {
        assert_eq!(
            plugins_dir_from(os("/tmp/data"), os("/home/u")),
            Some(PathBuf::from("/tmp/data/gband/plugins"))
        );
    }

    #[test]
    fn data_home_fallback() {
        let expected = Some(PathBuf::from("/home/u/.local/share/gband/plugins"));
        assert_eq!(plugins_dir_from(None, os("/home/u")), expected);
        assert_eq!(plugins_dir_from(os("relative"), os("/home/u")), expected);
        assert_eq!(plugins_dir_from(os(""), os("/home/u")), expected);
        assert_eq!(plugins_dir_from(None, None), None);
        assert_eq!(plugins_dir_from(os("relative"), None), None);
    }

    #[test]
    fn files_derive_from_the_directory() {
        let dir = Path::new("/tmp/cfg/gband");
        assert_eq!(
            user_file(dir),
            PathBuf::from("/tmp/cfg/gband/user/init.lua")
        );
        assert_eq!(
            defaults_file(dir),
            PathBuf::from("/tmp/cfg/gband/defaults/init.lua")
        );
    }

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("gband-lua-directory-{name}-{}", std::process::id()));
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
            let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn first_run() {
        let scratch = Scratch::new("first");
        let dir = scratch.dir();
        prepare(&dir).unwrap();
        assert_eq!(fs::read_to_string(defaults_file(&dir)).unwrap(), DEFAULTS);
        assert_eq!(entries(&dir.join("defaults")), ["init.lua"]);
        assert!(entries(&dir.join("user")).is_empty());
    }

    #[test]
    fn missing_user_directory() {
        let scratch = Scratch::new("user");
        let dir = scratch.dir();
        fs::create_dir_all(dir.join("defaults")).unwrap();
        fs::write(defaults_file(&dir), DEFAULTS).unwrap();
        prepare(&dir).unwrap();
        assert!(entries(&dir.join("user")).is_empty());
    }

    #[test]
    fn edited_defaults_are_restored() {
        let scratch = Scratch::new("edited");
        let dir = scratch.dir();
        prepare(&dir).unwrap();
        fs::write(defaults_file(&dir), "gband.set { prefix = 'ctrl+b' }").unwrap();
        prepare(&dir).unwrap();
        assert_eq!(fs::read_to_string(defaults_file(&dir)).unwrap(), DEFAULTS);
        assert_eq!(entries(&dir.join("defaults")), ["init.lua"]);
    }

    #[test]
    fn user_files_are_kept() {
        let scratch = Scratch::new("kept");
        let dir = scratch.dir();
        fs::create_dir_all(dir.join("user")).unwrap();
        fs::write(user_file(&dir), "gband.unbind('prefix q')").unwrap();
        fs::write(dir.join("user").join("notes.txt"), "notes").unwrap();
        prepare(&dir).unwrap();
        assert_eq!(
            fs::read_to_string(user_file(&dir)).unwrap(),
            "gband.unbind('prefix q')"
        );
        assert_eq!(
            fs::read_to_string(dir.join("user").join("notes.txt")).unwrap(),
            "notes"
        );
    }

    #[test]
    fn matching_defaults_are_not_rewritten() {
        let scratch = Scratch::new("unchanged");
        let dir = scratch.dir();
        prepare(&dir).unwrap();
        let modified = || {
            fs::metadata(defaults_file(&dir))
                .unwrap()
                .modified()
                .unwrap()
        };
        let before = modified();
        std::thread::sleep(Duration::from_millis(20));
        prepare(&dir).unwrap();
        assert_eq!(modified(), before);
    }

    #[test]
    fn read_only_directory_fails() {
        let scratch = Scratch::new("read-only");
        fs::set_permissions(&scratch.0, fs::Permissions::from_mode(0o555)).unwrap();
        if fs::create_dir(scratch.0.join("probe")).is_ok() {
            return;
        }
        assert!(prepare(&scratch.dir()).is_err());
    }
}
