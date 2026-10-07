mod actions;
mod api;
pub mod bars;
mod border;
mod bridge;
mod bundled;
mod callbacks;
mod clock;
mod commands;
mod control;
mod directory;
mod error;
mod events;
mod guard;
mod keymap;
pub mod keys;
mod options;
mod owner;
pub mod plugin_windows;
pub mod removed;
mod runtime;
pub mod server;
mod sides;
pub mod ui;
mod value;
pub mod version;
mod watch;

use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind;
use std::path::Path;

pub use mlua::Lua;

pub use crate::actions::{ACTIONS, BuiltinAction};
pub use crate::api::{Binding, Chord, Dispatch, PluginWindowRequest, WindowInput};
pub use crate::bars::{Bar, BarSide, Columns, Slot};
pub use crate::border::{Border, BorderChars, CharSet, Sides};
pub use crate::bridge::{base64, clipboard_sequence, notification};
pub use crate::callbacks::CallbackId;
pub use crate::clock::freeze as freeze_time;
pub use crate::directory::{
    Locations, config_dir, config_dir_from, defaults_file, key_style_file, plugins_dir,
    plugins_dir_from, prepare, user_dir, user_file,
};
pub use crate::error::ConfigError;
pub use crate::events::{Event, Pointer, PointerTarget};
pub use crate::options::{NotifyStyle, OptValue, Options};
pub use crate::runtime::{API_VERSION, Outcome, Runtime};
pub use crate::sides::install_test;
pub use crate::ui::{
    BandState, ClientStyles, Color, PALETTE_COLORS, Palette, Rgb, Style, ViewState, WindowName,
    WindowNames, WindowStates,
};
pub use crate::version::{Requirement, Version};
pub use crate::watch::{Watcher, watch};

pub mod plain {
    pub use crate::value::{from_lua, into_lua};
}

pub const DEFAULTS: &str = include_str!("defaults.lua");
pub const DEFAULTS_SERVER: &str = include_str!("defaults_server.lua");
pub const KEY_STYLES: [(&str, &str); 2] = bundled::KEY_STYLES;
pub const BUDGET: u64 = 100_000_000;
const TEST_SIDE: &str = "the test side loads no configuration and has no counterpart";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Client,
    Server,
    Test,
}

impl Side {
    pub fn name(self) -> &'static str {
        match self {
            Side::Client => "client",
            Side::Server => "server",
            Side::Test => "test",
        }
    }

    pub fn other(self) -> Side {
        match self {
            Side::Client => Side::Server,
            Side::Server => Side::Client,
            Side::Test => unreachable!("{TEST_SIDE}"),
        }
    }

    pub fn init_name(self) -> &'static str {
        match self {
            Side::Client => "init.lua",
            Side::Server => "server.lua",
            Side::Test => unreachable!("{TEST_SIDE}"),
        }
    }

    pub fn file_name(self) -> &'static str {
        match self {
            Side::Client => "client.lua",
            Side::Server => "server.lua",
            Side::Test => unreachable!("{TEST_SIDE}"),
        }
    }

    pub fn defaults(self) -> &'static str {
        match self {
            Side::Client => DEFAULTS,
            Side::Server => DEFAULTS_SERVER,
            Side::Test => unreachable!("{TEST_SIDE}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadOptions {
    pub budget: u64,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self { budget: BUDGET }
    }
}

pub type KeyTables = BTreeMap<String, Vec<(Chord, Binding)>>;
pub type Modes = BTreeSet<String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginManifest {
    pub name: String,
    pub version: Option<String>,
    pub client: Option<String>,
    pub failed: bool,
}

pub struct Config {
    pub side: Side,
    pub options: Options,
    pub keymap: KeyTables,
    pub modes: Modes,
    pub runtime: Runtime,
    pub errors: Vec<ConfigError>,
    pub plugins: Vec<PluginManifest>,
}

pub fn load(
    locations: &Locations,
    side: Side,
    options: &LoadOptions,
) -> Result<Config, ConfigError> {
    let path = user_file(&locations.config, side);
    match std::fs::read(&path) {
        Ok(source) => evaluate(Some(locations), side, options, &path, &source),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            load_defaults(locations, side, options)
        }
        Err(error) => Err(ConfigError::new(format!(
            "cannot read {}: {error}",
            path.display()
        ))),
    }
}

pub fn load_defaults(
    locations: &Locations,
    side: Side,
    options: &LoadOptions,
) -> Result<Config, ConfigError> {
    evaluate(
        Some(locations),
        side,
        options,
        &defaults_file(&locations.config, side),
        side.defaults().as_bytes(),
    )
}

pub fn defaults(side: Side) -> Config {
    evaluate(
        None,
        side,
        &LoadOptions::default(),
        &Path::new("defaults").join(side.init_name()),
        side.defaults().as_bytes(),
    )
    .expect("the default configuration loads")
}

fn evaluate(
    locations: Option<&Locations>,
    side: Side,
    options: &LoadOptions,
    path: &Path,
    source: &[u8],
) -> Result<Config, ConfigError> {
    let lua = Lua::new();
    runtime::install(&lua, side, locations, options.budget)
        .map_err(|error| ConfigError::from_lua(&error, &[]))?;
    runtime::run_init(&lua, path, source)?;
    let user = locations.map(|locations| user_dir(&locations.config));
    runtime::source_plugins(&lua, user.as_deref())?;
    runtime::finish(lua)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_in_the_default_text_name_the_defaults_file() {
        let path = defaults_file(Path::new("/home/u/.config/gband"), Side::Client);
        let Err(error) = evaluate(
            None,
            Side::Client,
            &LoadOptions::default(),
            &path,
            b"\nerror('boom')",
        ) else {
            panic!("the text loaded");
        };
        assert_eq!(error.location, Some((path, 2)));
    }
}
