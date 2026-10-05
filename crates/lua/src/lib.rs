mod api;
mod directory;
mod error;
pub mod keys;
mod options;
mod watch;

use std::io::ErrorKind;
use std::path::Path;

pub use mlua::{Lua, RegistryKey};

pub use crate::api::{ACTIONS, Binding, Chord, Dispatch, Keys, call};
pub use crate::directory::{config_dir, config_dir_from, defaults_file, prepare, user_file};
pub use crate::error::ConfigError;
pub use crate::options::Options;
pub use crate::watch::{Watcher, watch};

use crate::api::{Loading, Source};

pub const DEFAULTS: &str = include_str!("defaults.lua");

pub struct Config {
    pub options: Options,
    pub bindings: Vec<(Keys, Binding)>,
    pub lua: Lua,
}

pub fn load(dir: &Path) -> Result<Config, ConfigError> {
    let path = user_file(dir);
    match std::fs::read(&path) {
        Ok(source) => evaluate(&path, &source),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            evaluate(&defaults_file(dir), DEFAULTS.as_bytes())
        }
        Err(error) => Err(ConfigError::new(format!(
            "cannot read {}: {error}",
            path.display()
        ))),
    }
}

pub fn defaults() -> Config {
    evaluate(Path::new("defaults/init.lua"), DEFAULTS.as_bytes())
        .expect("the default configuration loads")
}

fn evaluate(path: &Path, source: &[u8]) -> Result<Config, ConfigError> {
    let failed = |error: mlua::Error| ConfigError::from_lua(&error, Some(path));
    let lua = Lua::new();
    api::install(&lua).map_err(failed)?;
    lua.set_app_data(Source(path.to_path_buf()));
    lua.set_app_data(Loading::default());
    lua.load(source)
        .set_name(format!("@{}", path.display()))
        .exec()
        .map_err(failed)?;
    let loading = lua
        .remove_app_data::<Loading>()
        .expect("installed before evaluation");
    let options = loading.options;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_in_the_default_text_name_the_defaults_file() {
        let path = defaults_file(Path::new("/home/u/.config/gband"));
        let Err(error) = evaluate(&path, b"\nerror('boom')") else {
            panic!("the text loaded");
        };
        assert_eq!(error.location, Some((path, 2)));
    }
}
