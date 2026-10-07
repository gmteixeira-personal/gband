use std::collections::{BTreeSet, HashSet};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::time::Instant;

use gband_core::geometry::Size;
use gband_core::input::{Key, KeyCode, MouseButton, WheelDirection};
use gband_core::layout::WindowId;
use gband_protocol::Value as Data;

use mlua::{Function, IntoLuaMulti, Lua, MultiValue, Table, Value};

use crate::api::{self, Dispatch, Queue};
use crate::callbacks::{self, CallbackId, Callbacks, Ran};
use crate::error::{ConfigError, caller};
use crate::events::{self, Event, Pointer};
use crate::guard::{self, Failure};
use crate::keys::key_name;
use crate::owner::{self, Owners};
use crate::plugin_windows::{self, Frame, PluginBox, PluginMouse};
use crate::removed;
use crate::server::{self, Caller, Host};
use crate::ui::{self, ViewState};
use crate::version::{Requirement, Version};
use crate::{
    Config, Locations, PluginManifest, Side, actions, bridge, bundled, clock, commands, control,
    keymap, options, sides, user_dir, value,
};

pub const API_VERSION: i64 = 2;

pub(crate) fn side(lua: &Lua) -> Side {
    *lua.app_data_ref::<Side>()
        .expect("the side is installed with the runtime")
}

pub(crate) struct Phase {
    pub loading: bool,
}

pub(crate) fn is_loading(lua: &Lua) -> bool {
    lua.app_data_ref::<Phase>()
        .is_some_and(|phase| phase.loading)
}

#[derive(Default)]
struct SetUp(BTreeSet<String>);

#[derive(Default)]
pub(crate) struct Manifests(Vec<PluginManifest>);

pub(crate) fn install(
    lua: &Lua,
    side: Side,
    locations: Option<&Locations>,
    budget: u64,
) -> mlua::Result<()> {
    lua.set_app_data(side);
    lua.set_app_data(Phase { loading: true });
    lua.set_app_data(Owners::default());
    lua.set_app_data(Callbacks::default());
    lua.set_app_data(SetUp::default());
    lua.set_app_data(Manifests::default());
    guard::install(lua, budget)?;
    let gband = lua.create_table()?;
    api::install(lua, &gband, side)?;
    actions::install(lua, &gband, side)?;
    commands::install(lua, &gband)?;
    events::install(lua, &gband, side)?;
    options::install(lua, &gband)?;
    gband.set("side", side.name())?;
    gband.set("api_version", API_VERSION)?;
    gband.set(
        "runtimepath",
        lua.create_sequence_from(
            runtimepath(locations)
                .iter()
                .map(|entry| entry.to_string_lossy().into_owned()),
        )?,
    )?;
    gband.set(
        "config_dir",
        locations.map(|locations| locations.config.to_string_lossy().into_owned()),
    )?;
    gband.set("plugin", lua.create_function(plugin)?)?;
    gband.set("plugins", lua.create_function(plugins)?)?;
    match side {
        Side::Client => {
            keymap::install(lua, &gband)?;
            control::install(lua, &gband)?;
            bridge::install(lua, &gband)?;
            let core = ui::install(lua, &gband)?;
            core.set(
                "colorschemes",
                lua.create_function(|lua, ()| colorschemes(lua))?,
            )?;
            core.set(
                "bundled_themes",
                lua.create_sequence_from(bundled::themes())?,
            )?;
            plugin_windows::install(lua, &core)?;
            crate::bars::install(lua, &core)?;
            removed::install(lua, &core)?;
        }
        Side::Server => server::install(lua, &gband)?,
        Side::Test => {}
    }
    lua.globals().set("gband", gband.clone())?;
    lua.globals().set("print", lua.create_function(print)?)?;
    clock::install(lua)?;
    let searchers: Table = lua.globals().get::<Table>("package")?.get("searchers")?;
    let insert: Function = lua.globals().get::<Table>("table")?.get("insert")?;
    insert.call::<()>((searchers, 2, lua.create_function(search)?))?;
    bundled::install_searcher(lua, 3)?;
    if side == Side::Client {
        let require: Function = lua.globals().get("require")?;
        match guard::run(lua, None, || require.call::<()>("gband.prelude"))? {
            Ok(()) => {}
            Err(failure) => return Err(mlua::Error::external(failure.error)),
        }
    }
    sides::guard(lua, &gband, side)
}

fn colorschemes(lua: &Lua) -> mlua::Result<Vec<String>> {
    let mut names = BTreeSet::new();
    for entry in current_runtimepath(lua)? {
        let Ok(listing) = fs::read_dir(entry.join("colors")) else {
            continue;
        };
        for file in listing.filter_map(Result::ok) {
            let path = file.path();
            if path.extension().is_some_and(|extension| extension == "lua")
                && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
            {
                names.insert(stem.to_owned());
            }
        }
    }
    Ok(names.into_iter().collect())
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

pub(crate) fn load_file(lua: &Lua, path: &Path) -> mlua::Result<Function> {
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

const MANIFEST: &str = "plugin.lua";

fn plugin_error(lua: &Lua, plugin: &str, location: Option<(PathBuf, u32)>, message: String) {
    guard::report(
        lua,
        Failure {
            error: ConfigError {
                plugin: Some(plugin.to_owned()),
                location,
                message,
            },
            limit: false,
        },
    );
}

fn manifest(lua: &Lua, name: &str, path: &Path) -> mlua::Result<PluginManifest> {
    let mut record = PluginManifest {
        name: name.to_owned(),
        version: None,
        client: None,
        failed: false,
    };
    let ran = guard::run(lua, Some(name.to_owned()), || {
        let source = fs::read(path).map_err(|error| {
            mlua::Error::runtime(format!("cannot read {}: {error}", path.display()))
        })?;
        guard::add_source(lua, path.to_path_buf());
        lua.load(source)
            .set_name(format!("@{}", path.display()))
            .set_environment(lua.create_table()?)
            .call::<Value>(())
    })?;
    let returned = match ran {
        Ok(returned) => returned,
        Err(failure) => {
            guard::report(lua, failure);
            return Ok(record);
        }
    };
    let location = Some((path.to_path_buf(), 1));
    let invalid = |message: String, record: PluginManifest| {
        plugin_error(lua, name, location.clone(), message);
        Ok(record)
    };
    let Value::Table(table) = returned else {
        return invalid(
            format!("the manifest of `{name}` must return a table holding `name` and `version`"),
            record,
        );
    };
    let text = |field: &str| -> mlua::Result<Result<Option<String>, String>> {
        Ok(match table.raw_get::<Value>(field)? {
            Value::Nil => Ok(None),
            Value::String(text) => Ok(Some(text.to_str()?.to_owned())),
            other => Err(format!(
                "the `{field}` of the manifest of `{name}` must be a string, found {}",
                other.type_name()
            )),
        })
    };
    let declared = match text("name")? {
        Ok(Some(declared)) => declared,
        Ok(None) => {
            return invalid(
                format!("the manifest of `{name}` must hold a `name`"),
                record,
            );
        }
        Err(message) => return invalid(message, record),
    };
    if declared != name {
        return invalid(
            format!(
                "the manifest of `{name}` names the plugin `{declared}`; it must name `{name}`"
            ),
            record,
        );
    }
    match text("version")? {
        Ok(Some(version)) => record.version = Some(version),
        Ok(None) => {
            return invalid(
                format!("the manifest of `{name}` must hold a `version`"),
                record,
            );
        }
        Err(message) => return invalid(message, record),
    }
    if let Some(Err(reason)) = record.version.as_deref().map(str::parse::<Version>) {
        return invalid(format!("the manifest of `{name}`: {reason}"), record);
    }
    match text("client")? {
        Ok(client) => record.client = client,
        Err(message) => return invalid(message, record),
    }
    if let Some(Err(reason)) = record.client.as_deref().map(str::parse::<Requirement>) {
        return invalid(format!("the manifest of `{name}`: {reason}"), record);
    }
    Ok(record)
}

pub(crate) fn source_plugins(lua: &Lua, user: Option<&Path>) -> Result<(), ConfigError> {
    let fail = |error: mlua::Error| ConfigError::from_lua(&error, &guard::sources(lua));
    let entries = current_runtimepath(lua).map_err(fail)?;
    let side = side(lua);
    let mut seen = HashSet::new();
    let mut plugins = Vec::new();
    for entry in entries {
        if Some(entry.as_path()) == user || !seen.insert(entry.clone()) {
            continue;
        }
        let Some(name) = entry
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
        else {
            continue;
        };
        let path = entry.join(MANIFEST);
        if !path.is_file() {
            let orphans: Vec<&str> = [Side::Client, Side::Server]
                .into_iter()
                .map(Side::file_name)
                .filter(|file| entry.join(file).is_file())
                .collect();
            if !orphans.is_empty() {
                plugin_error(
                    lua,
                    &name,
                    None,
                    format!(
                        "{} holds {} but no {MANIFEST} manifest, so the plugin `{name}` is not loaded",
                        entry.display(),
                        orphans.join(" and ")
                    ),
                );
            }
            continue;
        }
        let manifest = manifest(lua, &name, &path).map_err(fail)?;
        lua.app_data_mut::<Manifests>()
            .expect("manifests are installed with the runtime")
            .0
            .push(manifest);
        plugins.push((entry, name));
    }
    for (entry, name) in plugins {
        let file = entry.join(side.file_name());
        if owner::is_failed(lua, Some(&name)) || !file.is_file() {
            continue;
        }
        let ran = guard::run(lua, Some(name), || load_file(lua, &file)?.call::<()>(()));
        match ran {
            Ok(Ok(())) => {}
            Ok(Err(failure)) => guard::report(lua, failure),
            Err(error) => return Err(fail(error)),
        }
    }
    Ok(())
}

fn manifests(lua: &Lua) -> Vec<PluginManifest> {
    lua.app_data_ref::<Manifests>()
        .expect("manifests are installed with the runtime")
        .0
        .iter()
        .map(|manifest| PluginManifest {
            failed: owner::is_failed(lua, Some(&manifest.name)),
            ..manifest.clone()
        })
        .collect()
}

fn plugins(lua: &Lua, (): ()) -> mlua::Result<Table> {
    let list = lua.create_table()?;
    for manifest in manifests(lua) {
        let entry = lua.create_table()?;
        entry.set("name", manifest.name)?;
        entry.set("version", manifest.version)?;
        entry.set("client", manifest.client)?;
        entry.set("failed", manifest.failed)?;
        list.push(entry)?;
    }
    Ok(list)
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
    let side = side(&lua);
    let (keymap, modes) = match side {
        Side::Client => keymap::finish(&lua, options.prefix)?,
        Side::Server | Side::Test => Default::default(),
    };
    lua.set_app_data(Phase { loading: false });
    let errors = guard::drain(&lua);
    let plugins = manifests(&lua);
    Ok(Config {
        side,
        options,
        keymap,
        modes,
        runtime: Runtime { lua },
        errors,
        plugins,
    })
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub dispatched: Vec<Dispatch>,
    pub errors: Vec<ConfigError>,
    pub disabled: bool,
    pub declined: bool,
}

enum Ending {
    Returned,
    Disabled,
    Declined,
}

impl<R> From<&Ran<R>> for Ending {
    fn from(ran: &Ran<R>) -> Self {
        match ran {
            Ran::Disabled => Ending::Disabled,
            Ran::Returned(_) | Ran::Failed(_) => Ending::Returned,
        }
    }
}

fn declining(ran: Ran<Value>) -> Ending {
    match ran {
        Ran::Returned(Value::Boolean(false)) => Ending::Declined,
        ran => Ending::from(&ran),
    }
}

pub struct Runtime {
    lua: Lua,
}

impl Runtime {
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    pub fn call(&self, callback: CallbackId) -> Outcome {
        self.within_callback(|lua| Ok(Ending::from(&callbacks::run::<()>(lua, callback, ())?)))
    }

    pub fn call_pressed(
        &self,
        callback: CallbackId,
        button: MouseButton,
        pointer: &Pointer,
    ) -> Outcome {
        self.within_callback(|lua| {
            let payload = pointer.pressed(lua, button)?;
            Ok(declining(callbacks::run(lua, callback, payload)?))
        })
    }

    pub fn call_scrolled(
        &self,
        callback: CallbackId,
        direction: WheelDirection,
        pointer: &Pointer,
    ) -> Outcome {
        self.within_callback(|lua| {
            let payload = pointer.scrolled(lua, direction)?;
            Ok(declining(callbacks::run(lua, callback, payload)?))
        })
    }

    pub fn emit(&self, event: &Event) -> Outcome {
        self.within_callback(|lua| {
            events::emit_event(lua, event)?;
            ui::after_event(lua, Some(event.name()))?;
            Ok(Ending::Returned)
        })
    }

    pub fn set_state(&self, state: ViewState) -> Outcome {
        self.within_callback(|lua| ui::set_state(lua, state).map(|()| Ending::Returned))
    }

    pub fn refresh_plugins(&self) -> Outcome {
        self.within_callback(|lua| ui::after_event(lua, None).map(|()| Ending::Returned))
    }

    pub fn next_timer(&self) -> Option<Instant> {
        ui::next_timer(&self.lua)
    }

    pub fn fire_timers(&self, now: Instant) -> Outcome {
        self.within_callback(|lua| ui::fire_timers(lua, now).map(|()| Ending::Returned))
    }

    pub fn plugin_window_key(&self, plugin_window: u32, key: Key) -> Outcome {
        self.within_callback(|lua| {
            plugin_windows::call::<()>(lua, "key", (plugin_window, key_name(key), typed(key)))
                .map(|()| Ending::Returned)
        })
    }

    pub fn plugin_window_mouse(&self, plugin_window: u32, mouse: &PluginMouse) -> Outcome {
        self.within_callback(|lua| {
            let event = mouse.to_lua(lua)?;
            plugin_windows::call::<()>(lua, "mouse", (plugin_window, event))
                .map(|()| Ending::Returned)
        })
    }

    pub fn set_plugin_window_box(&self, plugin_window: u32, placed: PluginBox) -> Outcome {
        self.within_callback(|lua| {
            plugin_windows::call::<()>(
                lua,
                "set_box",
                (
                    plugin_window,
                    placed.col,
                    placed.row,
                    placed.width,
                    placed.height,
                ),
            )
            .map(|()| Ending::Returned)
        })
    }

    pub fn raise_plugin_window(&self, plugin_window: u32) -> Outcome {
        self.within_callback(|lua| {
            plugin_windows::call::<()>(lua, "raise", plugin_window).map(|()| Ending::Returned)
        })
    }

    pub fn unfocus_plugin_windows(&self) -> Outcome {
        self.within_callback(|lua| {
            plugin_windows::call::<()>(lua, "unfocus", ()).map(|()| Ending::Returned)
        })
    }

    pub fn plugin_window_paste(&self, plugin_window: u32, text: &str) -> Outcome {
        self.within_callback(|lua| {
            plugin_windows::call::<()>(lua, "paste", (plugin_window, text))
                .map(|()| Ending::Returned)
        })
    }

    pub fn plugin_window_opened(&self, plugin_window: u32, window: Option<WindowId>) -> Outcome {
        self.within_callback(|lua| {
            plugin_windows::call::<()>(
                lua,
                "opened",
                (plugin_window, window.map(|window| window.0)),
            )
            .map(|()| Ending::Returned)
        })
    }

    pub fn window_resized(&self, plugin_window: u32, size: Size) -> Outcome {
        self.within_callback(|lua| {
            plugin_windows::call::<()>(lua, "window_resized", (plugin_window, size.cols, size.rows))
                .map(|()| Ending::Returned)
        })
    }

    pub fn window_closed(&self, plugin_window: u32) -> Outcome {
        self.within_callback(|lua| {
            plugin_windows::call::<()>(lua, "window_closed", plugin_window)
                .map(|()| Ending::Returned)
        })
    }

    pub fn close_focused_plugin_window(&self) -> (bool, Outcome) {
        let mut closed = false;
        let outcome = self.within_callback(|lua| {
            closed = plugin_windows::call::<bool>(lua, "close_focused", ())?;
            Ok(Ending::Returned)
        });
        (closed, outcome)
    }

    pub fn release_plugin_windows(&self) -> Outcome {
        self.within_callback(|lua| {
            plugin_windows::call::<()>(lua, "release", ()).map(|()| Ending::Returned)
        })
    }

    pub fn take_frames(&self) -> Vec<(u32, Option<Frame>)> {
        plugin_windows::take(&self.lua)
    }

    pub fn take_bars(&self) -> Option<Vec<crate::bars::Bar>> {
        crate::bars::take(&self.lua)
    }

    pub fn take_settings_reopen(&self) -> Option<u32> {
        ui::take_settings_reopen(&self.lua)
    }

    pub fn open_settings(&self, line: u32) -> Outcome {
        self.within_callback(|lua| ui::open_settings(lua, line).map(|()| Ending::Returned))
    }

    pub fn take_palette(&self) -> Option<ui::Palette> {
        ui::take_palette(&self.lua)
    }

    pub fn take_client_styles(&self) -> Option<ui::ClientStyles> {
        ui::take_client_styles(&self.lua).unwrap_or_else(|error| {
            tracing::warn!("cannot resolve the client groups: {error}");
            None
        })
    }

    pub fn error_marker_shown(&self) -> bool {
        crate::bars::error_marker_shown(&self.lua)
    }

    pub fn set_plugin_window_counter(&self, counter: Arc<AtomicU32>) {
        plugin_windows::set_counter(&self.lua, counter);
    }

    pub fn handles(&self, event: &str) -> bool {
        events::handles(&self.lua, event)
    }

    pub fn emit_server(&self, event: &server::Event) -> Outcome {
        self.within_callback(|lua| {
            events::emit_server(lua, event)?;
            Ok(Ending::Returned)
        })
    }

    pub fn set_host(&self, host: Arc<dyn Host>) {
        server::set_host(&self.lua, host);
    }

    pub fn command(
        &self,
        name: &str,
        args: Data,
        caller: Option<Caller>,
    ) -> (Result<Data, String>, Outcome) {
        let mut answer = Err(format!("the command `{name}` did not run"));
        let outcome = self.within_callback(|lua| {
            answer = commands::invoke(lua, name, &args, caller)?;
            Ok(Ending::Returned)
        });
        (answer, outcome)
    }

    pub fn eval(&self, source: &str, args: &[Data]) -> (Result<Vec<Data>, String>, Outcome) {
        let mut answer = Err("the chunk did not run".to_owned());
        let outcome = self.within_callback(|lua| {
            let ran = guard::run(lua, None, || {
                let chunk = lua.load(source).set_name("=chunk").into_function()?;
                let args = args
                    .iter()
                    .map(|arg| value::into_lua(lua, arg))
                    .collect::<mlua::Result<MultiValue>>()?;
                chunk.call::<MultiValue>(args)
            })?;
            answer = match ran {
                Ok(results) => results
                    .iter()
                    .enumerate()
                    .map(|(index, result)| {
                        value::from_lua(result, &format!("result {}", index + 1))
                    })
                    .collect(),
                Err(failure) => Err(failure.error.to_string()),
            };
            Ok(Ending::Returned)
        });
        (answer, outcome)
    }

    pub fn answer(&self, call: u64, result: Result<Data, String>) -> Outcome {
        self.within_callback(|lua| bridge::answer(lua, call, result).map(|()| Ending::Returned))
    }

    pub fn active_table(&self) -> String {
        keymap::active(&self.lua)
    }

    pub fn set_active_table(&self, table: &str) {
        keymap::set_active(&self.lua, table);
    }

    fn within_callback(&self, run: impl FnOnce(&Lua) -> mlua::Result<Ending>) -> Outcome {
        let lua = &self.lua;
        lua.app_data_mut::<Queue>()
            .expect("the queue is installed with the runtime")
            .0 = Some(Vec::new());
        let result = run(lua).and_then(|ending| match side(lua) {
            Side::Client => plugin_windows::flush(lua)
                .and_then(|()| crate::bars::flush(lua))
                .map(|()| ending),
            Side::Server | Side::Test => Ok(ending),
        });
        let dispatched = lua
            .app_data_mut::<Queue>()
            .and_then(|mut queue| queue.0.take())
            .unwrap_or_default();
        let mut errors = guard::drain(lua);
        let ending = result.unwrap_or_else(|error| {
            errors.push(ConfigError::from_lua(&error, &guard::sources(lua)));
            Ending::Returned
        });
        Outcome {
            dispatched,
            errors,
            disabled: matches!(ending, Ending::Disabled),
            declined: matches!(ending, Ending::Declined),
        }
    }
}

fn typed(key: Key) -> Option<String> {
    match key.code {
        KeyCode::Char(character) if !key.modifiers.ctrl && !key.modifiers.alt => {
            Some(character.to_string())
        }
        _ => None,
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
        install(&lua, Side::Client, None, crate::BUDGET).unwrap();
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
