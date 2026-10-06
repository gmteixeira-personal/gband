use std::collections::BTreeMap;
use std::path::PathBuf;

use gband_core::input::Key;
use mlua::{Lua, Table, Value};

use crate::actions::LuaAction;
use crate::api::{self, Binding, Chord, Dispatch, require_loading};
use crate::callbacks;
use crate::error::{ConfigError, caller};
use crate::guard::{self, Failure};
use crate::keys::parse_pressed;
use crate::owner;
use crate::{KeyTables, Modes};

pub(crate) const ROOT: &str = "root";
pub(crate) const PREFIX: &str = "prefix";

pub(crate) struct Target {
    binding: Binding,
    action: Option<String>,
}

struct Bound {
    table: String,
    key: String,
    chord: Chord,
    target: Target,
    desc: Option<String>,
    location: Option<(PathBuf, u32)>,
    owner: Option<String>,
}

pub(crate) struct Keymaps {
    bound: Vec<Bound>,
    modes: BTreeMap<String, Option<String>>,
    active: String,
}

impl Default for Keymaps {
    fn default() -> Self {
        Self {
            bound: Vec::new(),
            modes: BTreeMap::new(),
            active: ROOT.to_owned(),
        }
    }
}

fn keymaps(lua: &Lua) -> mlua::AppDataRefMut<'_, Keymaps> {
    lua.app_data_mut::<Keymaps>()
        .expect("the keymap is installed with the runtime")
}

pub(crate) fn install(lua: &Lua, gband: &Table) -> mlua::Result<()> {
    lua.set_app_data(Keymaps::default());
    let keymap = lua.create_table()?;
    keymap.set("set", lua.create_function(set)?)?;
    keymap.set("del", lua.create_function(del)?)?;
    keymap.set("list", lua.create_function(list)?)?;
    keymap.set("current_table", lua.create_function(current_table)?)?;
    keymap.set("enter", lua.create_function(enter)?)?;
    keymap.set("mode", lua.create_function(mode)?)?;
    keymap.set("label", lua.create_function(label)?)?;
    keymap.set("run", lua.create_function(run)?)?;
    gband.set("keymap", keymap)
}

pub(crate) fn target(lua: &Lua, value: &Value) -> mlua::Result<Option<Target>> {
    Ok(match value {
        Value::Function(function) => Some(Target {
            binding: Binding::Callback(callbacks::register(lua, function.clone())?),
            action: None,
        }),
        Value::UserData(data) => match data.borrow::<LuaAction>() {
            Ok(action) => match &*action {
                LuaAction::Builtin { name, action } => Some(Target {
                    binding: Binding::Action(*action),
                    action: Some((*name).to_owned()),
                }),
                LuaAction::Registered { name, callback } => Some(Target {
                    binding: Binding::Callback(*callback),
                    action: Some(name.clone()),
                }),
                LuaAction::Targeted { .. } => None,
            },
            Err(_) => None,
        },
        _ => None,
    })
}

pub(crate) fn insert(
    lua: &Lua,
    table: &str,
    key: String,
    chord: Chord,
    target: Target,
    desc: Option<String>,
) -> mlua::Result<()> {
    let location = caller(lua);
    let owner = owner::current(lua);
    let mut keymaps = keymaps(lua);
    keymaps
        .bound
        .retain(|bound| !(bound.table == table && bound.chord == chord));
    keymaps.bound.push(Bound {
        table: table.to_owned(),
        key,
        chord,
        target,
        desc,
        location,
        owner,
    });
    Ok(())
}

pub(crate) fn remove(lua: &Lua, table: &str, chord: Chord) {
    keymaps(lua)
        .bound
        .retain(|bound| !(bound.table == table && bound.chord == chord));
}

fn table_name(lua: &Lua, value: &Value, function: &str) -> mlua::Result<String> {
    match value {
        Value::String(name) if !name.as_bytes().is_empty() => Ok(name.to_str()?.to_owned()),
        _ => Err(ConfigError::raise(
            lua,
            format!("{function} expects a key table name as a non-empty string"),
        )),
    }
}

fn chord(lua: &Lua, table: &str, key: &Value, function: &str) -> mlua::Result<(String, Chord)> {
    let Value::String(text) = key else {
        return Err(ConfigError::raise(
            lua,
            format!("{function} expects a key name as a string"),
        ));
    };
    let text = text.to_str()?.to_owned();
    if text == PREFIX {
        if table == ROOT {
            return Err(ConfigError::raise(
                lua,
                "`prefix` cannot be bound in the root table",
            ));
        }
        return Ok((text, Chord::Prefix));
    }
    let pressed =
        parse_pressed(&text).map_err(|error| ConfigError::raise(lua, error.to_string()))?;
    Ok((text, Chord::from(pressed)))
}

fn description(lua: &Lua, opts: &Value, function: &str) -> mlua::Result<Option<String>> {
    let invalid = || ConfigError::raise(lua, format!("{function} expects `desc` to be a string"));
    match opts {
        Value::Nil => Ok(None),
        Value::Table(opts) => match opts.get::<Value>("desc")? {
            Value::Nil => Ok(None),
            Value::String(desc) => Ok(Some(desc.to_str()?.to_owned())),
            _ => Err(invalid()),
        },
        _ => Err(ConfigError::raise(
            lua,
            format!("{function} expects its options as a table"),
        )),
    }
}

fn set(lua: &Lua, (table, key, action, opts): (Value, Value, Value, Value)) -> mlua::Result<()> {
    let function = "gband.keymap.set";
    let table = table_name(lua, &table, function)?;
    let (text, chord) = chord(lua, &table, &key, function)?;
    let desc = description(lua, &opts, function)?;
    let target = target(lua, &action)?.ok_or_else(|| {
        ConfigError::raise(
            lua,
            format!(
                "the binding of `{text}` in `{table}` must be a gband.action value or a function"
            ),
        )
    })?;
    require_loading(lua, function)?;
    insert(lua, &table, text, chord, target, desc)
}

fn del(lua: &Lua, (table, key): (Value, Value)) -> mlua::Result<()> {
    let function = "gband.keymap.del";
    let table = table_name(lua, &table, function)?;
    let (_, chord) = chord(lua, &table, &key, function)?;
    require_loading(lua, function)?;
    remove(lua, &table, chord);
    Ok(())
}

fn list(lua: &Lua, table: Value) -> mlua::Result<Table> {
    let table = table_name(lua, &table, "gband.keymap.list")?;
    let entries = lua.create_table()?;
    let keymaps = keymaps(lua);
    for bound in keymaps.bound.iter().filter(|bound| bound.table == table) {
        let entry = lua.create_table()?;
        entry.set("key", bound.key.as_str())?;
        entry.set("desc", bound.desc.as_deref())?;
        entry.set("action", bound.target.action.as_deref())?;
        entries.push(entry)?;
    }
    Ok(entries)
}

fn current_table(lua: &Lua, (): ()) -> mlua::Result<String> {
    Ok(active(lua))
}

fn enter(lua: &Lua, table: Value) -> mlua::Result<()> {
    let function = "gband.keymap.enter";
    if !api::in_callback(lua) {
        return Err(api::outside_callback(lua, function));
    }
    let table = table_name(lua, &table, function)?;
    if table != ROOT && !keymaps(lua).bound.iter().any(|bound| bound.table == table) {
        return Err(ConfigError::raise(
            lua,
            format!("the key table `{table}` has no binding"),
        ));
    }
    api::queue(lua, Dispatch::Enter(table), function)
}

fn mode_label(lua: &Lua, opts: &Value, function: &str) -> mlua::Result<Option<String>> {
    let invalid = || {
        ConfigError::raise(
            lua,
            format!("{function} expects `label` to be a non-empty string"),
        )
    };
    match opts {
        Value::Nil => Ok(None),
        Value::Table(opts) => match opts.get::<Value>("label")? {
            Value::Nil => Ok(None),
            Value::String(label) if !label.as_bytes().is_empty() => {
                Ok(Some(label.to_str()?.to_owned()))
            }
            _ => Err(invalid()),
        },
        _ => Err(ConfigError::raise(
            lua,
            format!("{function} expects its options as a table"),
        )),
    }
}

fn mode(lua: &Lua, (table, opts): (Value, Value)) -> mlua::Result<()> {
    let function = "gband.keymap.mode";
    let table = table_name(lua, &table, function)?;
    if table == ROOT {
        return Err(ConfigError::raise(lua, "the root table cannot be a mode"));
    }
    let label = mode_label(lua, &opts, function)?;
    require_loading(lua, function)?;
    keymaps(lua).modes.insert(table, label);
    Ok(())
}

fn label(lua: &Lua, table: Value) -> mlua::Result<String> {
    let table = table_name(lua, &table, "gband.keymap.label")?;
    let label = keymaps(lua).modes.get(&table).cloned().flatten();
    Ok(label.unwrap_or(table))
}

fn run(lua: &Lua, (table, key): (Value, Value)) -> mlua::Result<bool> {
    let function = "gband.keymap.run";
    if !api::in_callback(lua) {
        return Err(api::outside_callback(lua, function));
    }
    let table = table_name(lua, &table, function)?;
    let (_, chord) = chord(lua, &table, &key, function)?;
    let binding = keymaps(lua)
        .bound
        .iter()
        .find(|bound| bound.table == table && bound.chord == chord)
        .map(|bound| bound.target.binding);
    match binding {
        None => Ok(false),
        Some(Binding::Action(action)) => {
            let action = action.stepped(crate::options::current(lua).steps);
            api::queue(lua, Dispatch::Action(action), function)?;
            Ok(true)
        }
        Some(Binding::Callback(callback)) => {
            callbacks::run::<()>(lua, callback, ())?;
            Ok(true)
        }
    }
}

pub(crate) fn active(lua: &Lua) -> String {
    keymaps(lua).active.clone()
}

pub(crate) fn set_active(lua: &Lua, table: &str) {
    keymaps(lua).active = table.to_owned();
}

pub(crate) fn finish(lua: &Lua, prefix: Key) -> Result<(KeyTables, Modes), ConfigError> {
    let clash = |bound: &Bound| bound.table == ROOT && bound.chord == Chord::Key(prefix);
    let clashes: Vec<ConfigError> = keymaps(lua)
        .bound
        .iter()
        .filter(|bound| clash(bound))
        .map(|bound| ConfigError {
            plugin: bound.owner.clone(),
            location: bound.location.clone(),
            message: "a root binding cannot use the prefix key".to_owned(),
        })
        .collect();
    for error in clashes {
        if error.plugin.is_none() {
            return Err(error);
        }
        guard::report(
            lua,
            Failure {
                error,
                limit: false,
            },
        );
    }
    let mut keymaps = keymaps(lua);
    keymaps.bound.retain(|bound| !clash(bound));
    let mut tables: KeyTables = BTreeMap::new();
    for bound in &keymaps.bound {
        tables
            .entry(bound.table.clone())
            .or_default()
            .push((bound.chord, bound.target.binding));
    }
    Ok((tables, keymaps.modes.keys().cloned().collect()))
}
