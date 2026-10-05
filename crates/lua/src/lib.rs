mod actions;
mod api;
mod callbacks;
mod commands;
mod directory;
mod error;
mod events;
mod guard;
mod keymap;
pub mod keys;
mod options;
mod owner;
mod runtime;
mod watch;

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

pub use mlua::Lua;

pub use crate::actions::{ACTIONS, BuiltinAction};
pub use crate::api::{Binding, Chord, Dispatch};
pub use crate::callbacks::CallbackId;
pub use crate::directory::{
    Locations, config_dir, config_dir_from, defaults_file, plugins_dir, plugins_dir_from, prepare,
    user_dir, user_file,
};
pub use crate::error::ConfigError;
pub use crate::events::Event;
pub use crate::options::{OptValue, Options};
pub use crate::runtime::{API_VERSION, Outcome, Runtime};
pub use crate::watch::{Watcher, watch};

pub const DEFAULTS: &str = include_str!("defaults.lua");
pub const BUDGET: u64 = 100_000_000;

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

pub struct Config {
    pub options: Options,
    pub keymap: KeyTables,
    pub runtime: Runtime,
    pub errors: Vec<ConfigError>,
}

pub fn load(locations: &Locations, options: &LoadOptions) -> Result<Config, ConfigError> {
    let path = user_file(&locations.config);
    match std::fs::read(&path) {
        Ok(source) => evaluate(Some(locations), options, &path, &source),
        Err(error) if error.kind() == ErrorKind::NotFound => load_defaults(locations, options),
        Err(error) => Err(ConfigError::new(format!(
            "cannot read {}: {error}",
            path.display()
        ))),
    }
}

pub fn load_defaults(locations: &Locations, options: &LoadOptions) -> Result<Config, ConfigError> {
    evaluate(
        Some(locations),
        options,
        &defaults_file(&locations.config),
        DEFAULTS.as_bytes(),
    )
}

pub fn defaults() -> Config {
    evaluate(
        None,
        &LoadOptions::default(),
        Path::new("defaults/init.lua"),
        DEFAULTS.as_bytes(),
    )
    .expect("the default configuration loads")
}

fn evaluate(
    locations: Option<&Locations>,
    options: &LoadOptions,
    path: &Path,
    source: &[u8],
) -> Result<Config, ConfigError> {
    let lua = Lua::new();
    runtime::install(&lua, locations, options.budget)
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
        let path = defaults_file(Path::new("/home/u/.config/gband"));
        let Err(error) = evaluate(None, &LoadOptions::default(), &path, b"\nerror('boom')") else {
            panic!("the text loaded");
        };
        assert_eq!(error.location, Some((path, 2)));
    }
}
