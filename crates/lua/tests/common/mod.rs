#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use gband_core::input::Key;
use gband_lua::keys::parse_key;
use gband_lua::{
    BandState, Binding, Chord, Config, ConfigError, LoadOptions, Locations, Outcome, Side,
    StatusLine, ViewState,
};
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
    ViewState {
        table: "root".to_owned(),
        band: BandState {
            number: 1,
            index: 1,
            count: 1,
        },
        column: None,
        window: None,
        width,
        drawn: true,
        error: None,
        ..ViewState::default()
    }
}

pub fn text(line: &StatusLine, width: u16) -> String {
    let mut cells = vec![" ".to_owned(); usize::from(width)];
    for span in &line.spans {
        let mut col = usize::from(span.col);
        for c in span.text.chars() {
            let cells_taken = gband_lua::ui::width(&c.to_string());
            if col < cells.len() {
                cells[col] = c.to_string();
            }
            for extra in 1..cells_taken {
                if col + extra < cells.len() {
                    cells[col + extra] = String::new();
                }
            }
            col += cells_taken;
        }
    }
    cells.concat()
}

pub fn presented(config: &Config, state: ViewState) -> StatusLine {
    clean(&config.runtime.set_state(state));
    clean(&config.runtime.refresh_statusline());
    config.runtime.take_line().expect("a line is presented")
}
