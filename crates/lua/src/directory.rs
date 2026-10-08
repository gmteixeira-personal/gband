use std::ffi::OsString;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{Side, bundled};

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

pub fn user_file(dir: &Path, side: Side) -> PathBuf {
    user_dir(dir).join(side.init_name())
}

pub fn defaults_file(dir: &Path, side: Side) -> PathBuf {
    dir.join("defaults").join(side.init_name())
}

pub fn key_style_file(dir: &Path, style: &str) -> PathBuf {
    dir.join("defaults")
        .join("keystyle")
        .join(format!("{style}.lua"))
}

pub fn prepare(dir: &Path) -> io::Result<()> {
    let defaults = dir.join("defaults");
    fs::create_dir_all(defaults.join("keystyle"))?;
    fs::create_dir_all(defaults.join("lua").join("gband"))?;
    fs::create_dir_all(defaults.join("colors"))?;
    fs::create_dir_all(dir.join("user"))?;
    write_defaults(&defaults_file(dir, Side::Client), Side::Client.defaults())?;
    for (style, content) in crate::KEY_STYLES {
        write_defaults(&key_style_file(dir, style), content)?;
    }
    for (file, content) in bundled::files().filter(|(file, _)| !bundled::is_key_style(file)) {
        let path = defaults.join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_defaults(&path, content)?;
    }
    write_defaults(&defaults_file(dir, Side::Server), Side::Server.defaults())
}

fn write_defaults(defaults: &Path, content: &str) -> io::Result<()> {
    match fs::read(defaults) {
        Ok(current) if current == content.as_bytes() => return Ok(()),
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    static WRITES: AtomicU64 = AtomicU64::new(0);
    let name = defaults
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temporary = defaults.with_file_name(format!(
        ".{name}.{}.{}",
        std::process::id(),
        WRITES.fetch_add(1, Ordering::Relaxed)
    ));
    let written = fs::write(&temporary, content).and_then(|()| fs::rename(&temporary, defaults));
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
            user_file(dir, Side::Client),
            PathBuf::from("/tmp/cfg/gband/user/init.lua")
        );
        assert_eq!(
            user_file(dir, Side::Server),
            PathBuf::from("/tmp/cfg/gband/user/server.lua")
        );
        assert_eq!(
            defaults_file(dir, Side::Client),
            PathBuf::from("/tmp/cfg/gband/defaults/init.lua")
        );
        assert_eq!(
            defaults_file(dir, Side::Server),
            PathBuf::from("/tmp/cfg/gband/defaults/server.lua")
        );
    }

    struct Scratch(gband_scratch::Scratch);

    impl Scratch {
        fn new(name: &str) -> Self {
            Self(gband_scratch::Scratch::new(
                "lua",
                &format!("directory-{name}"),
            ))
        }

        fn dir(&self) -> PathBuf {
            self.0.join("gband")
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
        for side in [Side::Client, Side::Server] {
            assert_eq!(
                fs::read_to_string(defaults_file(&dir, side)).unwrap(),
                side.defaults()
            );
        }
        for (style, content) in crate::KEY_STYLES {
            assert_eq!(
                fs::read_to_string(key_style_file(&dir, style)).unwrap(),
                content
            );
        }
        assert_eq!(
            entries(&dir.join("defaults")),
            ["colors", "init.lua", "keystyle", "lua", "server.lua"]
        );
        assert_eq!(
            entries(&dir.join("defaults").join("keystyle")),
            ["direct.lua", "floating.lua", "modal.lua"]
        );
        assert!(entries(&dir.join("user")).is_empty());
    }

    #[test]
    fn sources_written() {
        let scratch = Scratch::new("sources");
        let dir = scratch.dir();
        prepare(&dir).unwrap();
        let defaults = dir.join("defaults");
        for (file, content) in bundled::files() {
            let path = defaults.join(&file);
            if bundled::is_key_style(&file) {
                assert!(!path.exists(), "{}", path.display());
            } else {
                assert_eq!(fs::read_to_string(&path).unwrap(), content, "{file}");
            }
        }
        for module in ["sidebar.lua", "win.lua", "prelude.lua", "keylist.lua"] {
            assert!(defaults.join("lua").join("gband").join(module).is_file());
        }
        for theme in ["nord.lua", "gruvbox.lua"] {
            assert!(defaults.join("colors").join(theme).is_file());
        }
        assert!(
            defaults
                .join("lua")
                .join("gband")
                .join("theme")
                .join("catppuccin.lua")
                .is_file()
        );
    }

    #[test]
    fn missing_user_directory() {
        let scratch = Scratch::new("user");
        let dir = scratch.dir();
        fs::create_dir_all(dir.join("defaults")).unwrap();
        fs::write(defaults_file(&dir, Side::Client), crate::DEFAULTS).unwrap();
        prepare(&dir).unwrap();
        assert!(entries(&dir.join("user")).is_empty());
    }

    #[test]
    fn edited_defaults_are_restored() {
        let scratch = Scratch::new("edited");
        let dir = scratch.dir();
        prepare(&dir).unwrap();
        let sidebar = dir
            .join("defaults")
            .join("lua")
            .join("gband")
            .join("sidebar.lua");
        let edited = [
            defaults_file(&dir, Side::Client),
            key_style_file(&dir, "modal"),
            defaults_file(&dir, Side::Server),
            sidebar.clone(),
        ];
        for file in &edited {
            fs::write(file, "gband.set { prefix = 'ctrl+b' }").unwrap();
        }
        prepare(&dir).unwrap();
        for side in [Side::Client, Side::Server] {
            assert_eq!(
                fs::read_to_string(defaults_file(&dir, side)).unwrap(),
                side.defaults()
            );
        }
        assert_eq!(
            fs::read_to_string(key_style_file(&dir, "modal")).unwrap(),
            crate::KEY_STYLES[0].1
        );
        let bundled = bundled::files()
            .find(|(file, _)| file == "lua/gband/sidebar.lua")
            .unwrap()
            .1;
        assert_eq!(fs::read_to_string(&sidebar).unwrap(), bundled);
        assert_eq!(
            entries(&dir.join("defaults")),
            ["colors", "init.lua", "keystyle", "lua", "server.lua"]
        );
        assert_eq!(
            entries(&dir.join("defaults").join("keystyle")),
            ["direct.lua", "floating.lua", "modal.lua"]
        );
    }

    #[test]
    fn user_files_are_kept() {
        let scratch = Scratch::new("kept");
        let dir = scratch.dir();
        fs::create_dir_all(dir.join("user")).unwrap();
        fs::write(user_file(&dir, Side::Client), "gband.unbind('prefix q')").unwrap();
        fs::write(user_file(&dir, Side::Server), "gband.set {}").unwrap();
        fs::write(dir.join("user").join("notes.txt"), "notes").unwrap();
        prepare(&dir).unwrap();
        assert_eq!(
            fs::read_to_string(user_file(&dir, Side::Client)).unwrap(),
            "gband.unbind('prefix q')"
        );
        assert_eq!(
            fs::read_to_string(user_file(&dir, Side::Server)).unwrap(),
            "gband.set {}"
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
            fs::metadata(defaults_file(&dir, Side::Client))
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
