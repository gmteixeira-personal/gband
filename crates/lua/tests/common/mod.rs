#![allow(dead_code)]

pub mod docs;

use std::fs;
use std::path::{Path, PathBuf};

use gband_core::input::Key;
use gband_lua::keys::parse_key;
use gband_lua::{
    BandState, Bar, Binding, Chord, Config, ConfigError, LoadOptions, Locations, Outcome, Side,
    ViewState,
};
use mlua::FromLua;

pub struct Scratch(pub gband_scratch::Scratch);

impl Scratch {
    pub fn new(name: &str) -> Self {
        Self(gband_scratch::Scratch::new(
            "lua",
            &format!("{}-{name}", env!("CARGO_CRATE_NAME")),
        ))
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

    pub fn server(&self, source: &str) -> PathBuf {
        self.user_file("server.lua", source)
    }

    pub fn plugin(&self, name: &str, manifest: &str) -> PathBuf {
        self.plugin_file(name, "plugin.lua", manifest)
    }

    pub fn client_plugin(&self, name: &str, source: &str) -> PathBuf {
        self.plugin(name, &manifest(name));
        self.plugin_file(name, "client.lua", source)
    }

    pub fn user_file(&self, relative: &str, source: &str) -> PathBuf {
        write(&self.user().join(relative), source)
    }

    pub fn plugin_file(&self, plugin: &str, relative: &str, source: &str) -> PathBuf {
        write(&self.plugins().join(plugin).join(relative), source)
    }

    pub fn load(&self) -> Result<Config, ConfigError> {
        gband_lua::load(&self.locations(), Side::Client, &LoadOptions::default())
    }

    pub fn load_server(&self) -> Result<Config, ConfigError> {
        gband_lua::load(&self.locations(), Side::Server, &LoadOptions::default())
    }

    pub fn load_with_budget(&self, budget: u64) -> Result<Config, ConfigError> {
        gband_lua::load(&self.locations(), Side::Client, &LoadOptions { budget })
    }

    pub fn loaded(&self) -> Config {
        self.load().unwrap_or_else(|error| panic!("{error}"))
    }

    pub fn loaded_server(&self) -> Config {
        self.load_server().unwrap_or_else(|error| panic!("{error}"))
    }
}

pub fn write(path: &Path, source: &str) -> PathBuf {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
    path.to_path_buf()
}

pub fn manifest(name: &str) -> String {
    format!("return {{ name = '{name}', version = '0.1.0' }}")
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

pub const JOB: &str = "gband.bind('alt+x', function() job() end)\n";

pub fn run_job(config: &Config, code: &str) -> Outcome {
    config
        .runtime
        .lua()
        .load(format!("job = function()\n{code}\nend"))
        .exec()
        .unwrap_or_else(|error| panic!("{error}"));
    let chord = Chord::Key(key("alt+x"));
    let Some((_, Binding::Callback(callback))) = config.keymap["root"]
        .iter()
        .find(|(bound, _)| *bound == chord)
    else {
        panic!("alt+x is not bound to a function");
    };
    config.runtime.call(*callback)
}

pub fn drawn(width: u16) -> ViewState {
    sized(width, 24)
}

pub fn sized(width: u16, height: u16) -> ViewState {
    ViewState {
        table: "root".to_owned(),
        band: BandState {
            number: 1,
            index: 1,
            count: 1,
        },
        window: None,
        width,
        height,
        error: None,
        ..ViewState::default()
    }
}

pub fn rows(bar: &Bar) -> Vec<String> {
    bar.lines
        .iter()
        .map(|runs| runs.iter().map(|run| run.text.as_str()).collect())
        .collect()
}

pub fn shown_rows(bar: &Bar) -> Vec<(usize, String)> {
    rows(bar)
        .into_iter()
        .enumerate()
        .filter(|(_, row)| !row.is_empty())
        .collect()
}

pub fn sidebar(config: &Config) -> Option<Bar> {
    config
        .runtime
        .take_bars()?
        .into_iter()
        .find(|bar| bar.id == "sidebar")
}

pub fn presented(config: &Config, state: ViewState) -> Bar {
    clean(&config.runtime.set_state(state));
    clean(&config.runtime.refresh_plugins());
    sidebar(config).expect("the sidebar is presented")
}
