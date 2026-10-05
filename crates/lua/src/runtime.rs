use std::collections::{BTreeSet, HashSet};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use mlua::{Function, IntoLuaMulti, Lua, MultiValue, Table, Value};

use crate::api::{self, Dispatch, Queue};
use crate::callbacks::{self, CallbackId, Callbacks, Ran};
use crate::error::{ConfigError, caller};
use crate::events::{self, Event};
use crate::guard::{self, Failure};
use crate::owner::{self, Owners};
use crate::{Config, Locations, actions, commands, keymap, options, user_dir};

pub const API_VERSION: i64 = 1;
const SIDE: &str = "client";

pub(crate) struct Phase {
    pub loading: bool,
}

pub(crate) fn is_loading(lua: &Lua) -> bool {
    lua.app_data_ref::<Phase>()
        .is_some_and(|phase| phase.loading)
}

#[derive(Default)]
struct SetUp(BTreeSet<String>);

pub(crate) fn install(lua: &Lua, locations: Option<&Locations>, budget: u64) -> mlua::Result<()> {
    lua.set_app_data(Phase { loading: true });
    lua.set_app_data(Owners::default());
    lua.set_app_data(Callbacks::default());
    lua.set_app_data(SetUp::default());
    guard::install(lua, budget)?;
    let gband = lua.create_table()?;
    api::install(lua, &gband)?;
    actions::install(lua, &gband)?;
    commands::install(lua, &gband)?;
    events::install(lua, &gband)?;
    keymap::install(lua, &gband)?;
    options::install(lua, &gband)?;
    gband.set("side", SIDE)?;
    gband.set("api_version", API_VERSION)?;
    gband.set(
        "runtimepath",
        lua.create_sequence_from(
            runtimepath(locations)
                .iter()
                .map(|entry| entry.to_string_lossy().into_owned()),
        )?,
    )?;
    gband.set("plugin", lua.create_function(plugin)?)?;
    lua.globals().set("gband", gband)?;
    lua.globals().set("print", lua.create_function(print)?)?;
    let searchers: Table = lua.globals().get::<Table>("package")?.get("searchers")?;
    let insert: Function = lua.globals().get::<Table>("table")?.get("insert")?;
    insert.call::<()>((searchers, 2, lua.create_function(search)?))
}

fn runtimepath(locations: Option<&Locations>) -> Vec<PathBuf> {
    let Some(locations) = locations else {
        return Vec::new();
    };
    let mut entries = vec![user_dir(&locations.config)];
    if let Some(plugins) = &locations.plugins
        && let Ok(listing) = fs::read_dir(plugins)
    {
        let mut names: Vec<OsString> = listing
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.file_name())
            .collect();
        names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        entries.extend(names.into_iter().map(|name| plugins.join(name)));
    }
    entries
}

fn current_runtimepath(lua: &Lua) -> mlua::Result<Vec<PathBuf>> {
    let gband: Table = lua.globals().get("gband")?;
    let Value::Table(list) = gband.get::<Value>("runtimepath")? else {
        return Ok(Vec::new());
    };
    Ok(list
        .sequence_values::<Value>()
        .filter_map(|value| match value {
            Ok(Value::String(entry)) => Some(PathBuf::from(entry.to_string_lossy())),
            _ => None,
        })
        .collect())
}

fn load_file(lua: &Lua, path: &Path) -> mlua::Result<Function> {
    let source = fs::read(path).map_err(|error| {
        mlua::Error::runtime(format!("cannot read {}: {error}", path.display()))
    })?;
    guard::add_source(lua, path.to_path_buf());
    lua.load(source)
        .set_name(format!("@{}", path.display()))
        .into_function()
}

fn search(lua: &Lua, name: String) -> mlua::Result<MultiValue> {
    let relative = name.replace('.', "/");
    let mut tried = String::new();
    for entry in current_runtimepath(lua)? {
        let lua_dir = entry.join("lua");
        for candidate in [
            lua_dir.join(format!("{relative}.lua")),
            lua_dir.join(&relative).join("init.lua"),
        ] {
            if candidate.is_file() {
                let loader = load_file(lua, &candidate)?;
                return (loader, candidate.to_string_lossy().into_owned()).into_lua_multi(lua);
            }
            tried.push_str(&format!("\n\tno file '{}'", candidate.display()));
        }
    }
    tried.into_lua_multi(lua)
}

pub(crate) fn run_init(lua: &Lua, path: &Path, source: &[u8]) -> Result<(), ConfigError> {
    guard::add_source(lua, path.to_path_buf());
    let ran = guard::run(lua, None, || {
        lua.load(source)
            .set_name(format!("@{}", path.display()))
            .exec()
    });
    match ran {
        Ok(Ok(())) => Ok(()),
        Ok(Err(failure)) => Err(failure.error),
        Err(error) => Err(ConfigError::from_lua(&error, &guard::sources(lua))),
    }
}

fn plugin_files(entry: &Path) -> Vec<PathBuf> {
    let listed = |dir: PathBuf| {
        let mut files: Vec<PathBuf> = fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|file| file.path())
            .filter(|path| path.is_file() && path.as_os_str().as_bytes().ends_with(b".lua"))
            .collect();
        files.sort_by(|a, b| a.as_os_str().as_bytes().cmp(b.as_os_str().as_bytes()));
        files
    };
    let plugin = entry.join("plugin");
    let mut files = listed(plugin.clone());
    files.extend(listed(plugin.join("client")));
    files
}

pub(crate) fn source_plugins(lua: &Lua, user: Option<&Path>) -> Result<(), ConfigError> {
    let entries = current_runtimepath(lua)
        .map_err(|error| ConfigError::from_lua(&error, &guard::sources(lua)))?;
    let mut sourced = HashSet::new();
    for entry in entries {
        let owner = if Some(entry.as_path()) == user {
            None
        } else {
            entry
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        };
        for file in plugin_files(&entry) {
            if owner::is_failed(lua, owner.as_deref()) {
                break;
            }
            if !sourced.insert(file.clone()) {
                continue;
            }
            let ran = guard::run(lua, owner.clone(), || load_file(lua, &file)?.call::<()>(()));
            match ran {
                Ok(Ok(())) => {}
                Ok(Err(failure)) => guard::report(lua, failure),
                Err(error) => return Err(ConfigError::from_lua(&error, &guard::sources(lua))),
            }
        }
    }
    Ok(())
}

fn plugin(lua: &Lua, (name, opts): (Value, Value)) -> mlua::Result<bool> {
    let name = match &name {
        Value::String(name) if !name.as_bytes().is_empty() => name.to_str()?.to_owned(),
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.plugin expects a module name as a non-empty string",
            ));
        }
    };
    let location = caller(lua);
    let fail = |plugin: &str, message: String| {
        guard::report(
            lua,
            Failure {
                error: ConfigError {
                    plugin: Some(plugin.to_owned()),
                    location: location.clone(),
                    message,
                },
                limit: false,
            },
        );
        Ok(false)
    };
    let require: Function = lua.globals().get("require")?;
    let module = match guard::run(lua, Some(name.clone()), || {
        require.call::<Value>(name.as_str())
    })? {
        Ok(module) => module,
        Err(mut failure) => {
            if failure.error.location.is_none() {
                failure.error.location = location.clone();
                if let Some((first, _)) = failure.error.message.split_once('\n') {
                    failure.error.message = first.trim_end_matches(':').to_owned();
                }
            }
            guard::report(lua, failure);
            return Ok(false);
        }
    };
    let shape = || format!("the module `{name}` must be a table with a `setup` function");
    let Value::Table(module) = module else {
        return fail(&name, shape());
    };
    let Value::Function(setup) = module.get::<Value>("setup")? else {
        return fail(&name, shape());
    };
    let plugin = match module.get::<Value>("name")? {
        Value::Nil => name.clone(),
        Value::String(plugin) => plugin.to_str()?.to_owned(),
        _ => {
            return fail(
                &name,
                format!("the `name` of the module `{name}` must be a string"),
            );
        }
    };
    match module.get::<Value>("api")? {
        Value::Nil => {}
        Value::Integer(api) if api == API_VERSION => {}
        Value::Integer(api) => tracing::warn!(
            plugin = %plugin,
            "the plugin `{plugin}` is written for API version {api}, and gband provides version {API_VERSION}"
        ),
        _ => {
            return fail(
                &plugin,
                format!("the `api` of the plugin `{plugin}` must be an integer"),
            );
        }
    }
    let fresh = lua
        .app_data_mut::<SetUp>()
        .expect("installed with the runtime")
        .0
        .insert(plugin.clone());
    if !fresh {
        return fail(&plugin, format!("the plugin `{plugin}` is already set up"));
    }
    let opts = match opts {
        Value::Nil => Value::Table(lua.create_table()?),
        opts => opts,
    };
    match guard::run(lua, Some(plugin), || setup.call::<()>(opts))? {
        Ok(()) => Ok(true),
        Err(failure) => {
            guard::report(lua, failure);
            Ok(false)
        }
    }
}

fn print(lua: &Lua, values: MultiValue) -> mlua::Result<()> {
    let tostring: Function = lua.globals().get("tostring")?;
    let text = values
        .into_iter()
        .map(|value| tostring.call::<String>(value))
        .collect::<mlua::Result<Vec<_>>>()?
        .join("\t");
    match owner::current(lua) {
        Some(plugin) => tracing::info!(plugin = %plugin, "{text}"),
        None => tracing::info!("{text}"),
    }
    Ok(())
}

pub(crate) fn finish(lua: Lua) -> Result<Config, ConfigError> {
    options::finish(&lua);
    let options = options::current(&lua);
    let keymap = keymap::finish(&lua, options.prefix)?;
    lua.set_app_data(Phase { loading: false });
    let errors = guard::drain(&lua);
    Ok(Config {
        options,
        keymap,
        runtime: Runtime { lua },
        errors,
    })
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub dispatched: Vec<Dispatch>,
    pub errors: Vec<ConfigError>,
    pub disabled: bool,
}

pub struct Runtime {
    lua: Lua,
}

impl Runtime {
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    pub fn call(&self, callback: CallbackId) -> Outcome {
        self.within_callback(|lua| {
            Ok(matches!(
                callbacks::run::<()>(lua, callback, ())?,
                Ran::Disabled
            ))
        })
    }

    pub fn emit(&self, event: &Event) -> Outcome {
        self.within_callback(|lua| events::emit_event(lua, event).map(|()| false))
    }

    pub fn active_table(&self) -> String {
        keymap::active(&self.lua)
    }

    pub fn set_active_table(&self, table: &str) {
        keymap::set_active(&self.lua, table);
    }

    fn within_callback(&self, run: impl FnOnce(&Lua) -> mlua::Result<bool>) -> Outcome {
        let lua = &self.lua;
        lua.app_data_mut::<Queue>()
            .expect("the queue is installed with the runtime")
            .0 = Some(Vec::new());
        let result = run(lua);
        let dispatched = lua
            .app_data_mut::<Queue>()
            .and_then(|mut queue| queue.0.take())
            .unwrap_or_default();
        let mut errors = guard::drain(lua);
        let disabled = match result {
            Ok(disabled) => disabled,
            Err(error) => {
                errors.push(ConfigError::from_lua(&error, &guard::sources(lua)));
                false
            }
        };
        Outcome {
            dispatched,
            errors,
            disabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::process::Command;
    use std::sync::{Arc, Mutex};

    use super::*;

    const CHILD: &str = "GBAND_LUA_PRINT_CHILD";
    const MARKER: &str = "printed-by-the-plugin";

    fn installed() -> Lua {
        let lua = Lua::new();
        install(&lua, None, crate::BUDGET).unwrap();
        lua
    }

    fn print_as_plugin(lua: &Lua) {
        guard::run(lua, Some("hello".to_owned()), || {
            lua.load(format!("print('{MARKER}', 3)")).exec()
        })
        .unwrap()
        .unwrap();
    }

    #[derive(Clone, Default)]
    struct Captured(Arc<Mutex<Vec<u8>>>);

    impl Write for Captured {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn print_logs_the_text_and_the_plugin() {
        let captured = Captured::default();
        let writer = captured.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(move || writer.clone())
            .with_ansi(false)
            .finish();
        let lua = installed();
        tracing::subscriber::with_default(subscriber, || print_as_plugin(&lua));
        let log = String::from_utf8(captured.0.lock().unwrap().clone()).unwrap();
        assert!(log.contains("INFO"), "{log}");
        assert!(log.contains(&format!("{MARKER}\t3")), "{log}");
        assert!(log.contains("plugin=hello"), "{log}");
    }

    #[test]
    fn print_writes_nothing_to_stdout() {
        if std::env::var_os(CHILD).is_some() {
            print_as_plugin(&installed());
            return;
        }
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "runtime::tests::print_writes_nothing_to_stdout",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "{stdout}");
        assert!(stdout.contains("1 passed"), "{stdout}");
        assert!(!stdout.contains(MARKER), "{stdout}");
    }
}
