mod api;
mod error;
pub mod keys;
mod options;
mod watch;

use std::io::ErrorKind;
use std::path::Path;

pub use mlua::{Lua, RegistryKey};

pub use crate::api::{ACTIONS, Binding, Chord, Dispatch, Keys, call};
pub use crate::error::ConfigError;
pub use crate::options::Options;
pub use crate::watch::{Watcher, config_path, config_path_from, watch};

use crate::api::{Loading, Source};
use crate::error::DEFAULTS_CHUNK;

const DEFAULTS: &str = include_str!("defaults.lua");

pub struct Config {
    pub options: Options,
    pub bindings: Vec<(Keys, Binding)>,
    pub lua: Lua,
}

pub fn load(path: &Path) -> Result<Config, ConfigError> {
    let source = match std::fs::read(path) {
        Ok(source) => Some(source),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => {
            return Err(ConfigError::new(format!(
                "cannot read {}: {error}",
                path.display()
            )));
        }
    };
    evaluate(source.map(|source| (path, source)))
}

pub fn defaults() -> Config {
    evaluate(None).expect("defaults.lua loads")
}

fn evaluate(config: Option<(&Path, Vec<u8>)>) -> Result<Config, ConfigError> {
    let path = config.as_ref().map(|(path, _)| *path);
    let failed = |error: mlua::Error| ConfigError::from_lua(&error, path);
    let lua = Lua::new();
    api::install(&lua).map_err(failed)?;
    lua.set_app_data(Source(path.map(Path::to_path_buf)));
    lua.set_app_data(Loading::default());
    lua.load(DEFAULTS)
        .set_name(format!("@{DEFAULTS_CHUNK}"))
        .exec()
        .map_err(failed)?;
    if let Some((path, source)) = config {
        lua.load(source)
            .set_name(format!("@{}", path.display()))
            .exec()
            .map_err(failed)?;
    }
    let loading = lua
        .remove_app_data::<Loading>()
        .expect("installed before evaluation");
    let options = loading
        .options
        .complete()
        .ok_or_else(|| ConfigError::new("defaults.lua must set every option"))?;
    if let Some(bound) = loading
        .bindings
        .iter()
        .find(|bound| bound.keys == Keys::Direct(options.prefix))
    {
        return Err(ConfigError {
            location: bound.location.clone(),
            message: "a direct binding cannot use the prefix key".to_owned(),
        });
    }
    let bindings = loading
        .bindings
        .into_iter()
        .map(|bound| (bound.keys, bound.binding))
        .collect();
    Ok(Config {
        options,
        bindings,
        lua,
    })
}
