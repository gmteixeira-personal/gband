#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use gband_core::input::Key;
use gband_lua::keys::parse_key;
use gband_lua::{Config, ConfigError, LoadOptions, Locations, Outcome};
use mlua::FromLua;

pub struct Scratch(pub PathBuf);

impl Scratch {
    pub fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "gband-lua-{}-{name}-{}",
            env!("CARGO_CRATE_NAME"),
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    pub fn dir(&self) -> PathBuf {
        self.0.join("config").join("gband")
    }

    pub fn user(&self) -> PathBuf {
        self.dir().join("user")
    }

    pub fn plugins(&self) -> PathBuf {
        self.0.join("data").join("gband").join("plugins")
    }

    pub fn locations(&self) -> Locations {
        Locations {
            config: self.dir(),
            plugins: Some(self.plugins()),
        }
    }

    pub fn write(&self, source: &str) -> PathBuf {
        self.user_file("init.lua", source)
    }

    pub fn user_file(&self, relative: &str, source: &str) -> PathBuf {
        write(&self.user().join(relative), source)
    }

    pub fn plugin_file(&self, plugin: &str, relative: &str, source: &str) -> PathBuf {
        write(&self.plugins().join(plugin).join(relative), source)
    }

    pub fn load(&self) -> Result<Config, ConfigError> {
        gband_lua::load(&self.locations(), &LoadOptions::default())
    }

    pub fn load_with_budget(&self, budget: u64) -> Result<Config, ConfigError> {
        gband_lua::load(&self.locations(), &LoadOptions { budget })
    }

    pub fn loaded(&self) -> Config {
        self.load().unwrap_or_else(|error| panic!("{error}"))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn write(path: &Path, source: &str) -> PathBuf {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
    path.to_path_buf()
}

pub fn key(name: &str) -> Key {
    parse_key(name).unwrap()
}

pub fn eval<T: FromLua>(config: &Config, source: &str) -> T {
    config
        .runtime
        .lua()
        .load(source)
        .eval()
        .unwrap_or_else(|error| panic!("{error}"))
}

pub fn global<T: FromLua>(config: &Config, name: &str) -> T {
    config.runtime.lua().globals().get(name).unwrap()
}

pub fn assert_error_at(error: &ConfigError, path: &Path, line: u32, mentions: &str) {
    assert_eq!(error.location, Some((path.to_path_buf(), line)), "{error}");
    assert!(error.message.contains(mentions), "{error}");
}

pub fn clean(outcome: &Outcome) {
    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    assert!(!outcome.disabled);
}
